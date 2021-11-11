use crate::helper::{never, Terminator};
use crate::scope::{BlockScope, Label, Labels};
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::*;
use inkwell::module::Module;
use inkwell::targets::*;
use inkwell::types::*;
use inkwell::values::*;
use inkwell::OptimizationLevel;
use std::collections::HashMap;
use std::ops::Deref;
use x_lang_ast::node::Node;
use x_lang_ast::shared::{Kind, KindName};

pub struct Compiler<'ctx> {
    pub context: &'ctx Context,
    pub builder: Builder<'ctx>,
    pub module: Module<'ctx>,
    pub scope: BlockScope<'ctx>,
    pub labels: Labels<'ctx>,
    pub execution_engine: ExecutionEngine<'ctx>,
    pub print_fns: HashMap<&'static str, FunctionValue<'ctx>>,
    pub main_fn: Option<FunctionValue<'ctx>>,
    pub current_fn: Option<FunctionValue<'ctx>>,
    pub is_debug: bool,
}

impl<'ctx> Compiler<'ctx> {
    pub fn compile(ast: &Node, is_debug: bool) {
        let context = &Context::create();
        let module = context.create_module("main");
        let builder = context.create_builder();
        let scope = BlockScope::new();
        let labels = Labels::new();
        let execution_engine = module
            .create_jit_execution_engine(OptimizationLevel::None)
            .unwrap();

        let mut compiler = Compiler {
            context,
            module,
            builder,
            scope,
            labels,
            execution_engine,
            main_fn: None,
            current_fn: None,
            print_fns: HashMap::new(),
            is_debug,
        };

        // 开始编译
        compiler.compile_program(ast);

        #[cfg(not(test))]
        if is_debug {
            compiler.module.print_to_file(".debug.ll");
        }

        unsafe {
            compiler
                .execution_engine
                .run_function(compiler.main_fn.unwrap(), &vec![]);
        }

        // Target::initialize_all(&InitializationConfig::default());
        // let target = Target::from_name("x86-64").unwrap();
        // let target_machine = target
        //     .create_target_machine(
        //         &TargetMachine::get_default_triple(),
        //         "x86-64",
        //         "+avx2",
        //         OptimizationLevel::Default,
        //         RelocMode::Default,
        //         CodeModel::Default,
        //     )
        //     .unwrap();
        // target_machine.write_to_file(
        //     module,
        //     FileType::Object,
        //     Path::new("abc"),
        // );
    }

    pub fn compile_program(&mut self, node: &Node) {
        self.inject_build_in();

        match node {
            Node::Program { body } => {
                self.main_fn = Some(self.compile_function(
                    "main",
                    &vec![],
                    &vec![],
                    body,
                    &Kind::create("void"),
                    true,
                ));
            }
            _ => never(),
        };
    }

    // 编译一条语句，返回语句中是否被终结了
    pub fn compile_statement(&mut self, node: &Node) -> Terminator {
        match node {
            Node::FunctionDeclaration {
                id,
                arguments,
                body,
                return_kind,
            } => {
                let (name, ..) = id.deref().read_identifier();
                let args = arguments.iter();
                let args = args.map(|arg| {
                    let (.., kind) = arg.deref().read_identifier();
                    let kind_name = kind.read_kind_name().unwrap();
                    (match kind_name {
                        KindName::Number => self.build_number_type().into(),
                        KindName::Boolean => self.build_bool_type().into(),
                        KindName::Void => never(),
                    })
                });
                let args = args.collect();

                let body = match body.deref() {
                    Node::BlockStatement { body } => body,
                    _ => never(),
                };
                self.compile_function(name, &args, arguments, body, return_kind, false);
                Terminator::None
            }
            Node::VariableDeclaration { id, init } => {
                self.compile_variable_statement(id.deref(), init.deref());
                Terminator::None
            }
            Node::BlockStatement { body } => self.compile_block_statement(body, true),
            Node::ReturnStatement { argument } => {
                self.compile_return_statement(argument);
                Terminator::Return
            }
            Node::ExpressionStatement { expression } => {
                self.compile_expression(expression.deref());
                Terminator::None
            }
            Node::IfStatement {
                condition,
                consequent,
                alternate,
            } => self.compile_if_statement(condition, consequent, alternate),
            Node::LoopStatement { label, body } => {
                self.compile_loop_statement(label, body.deref())
            }
            Node::BreakStatement { label } => {
                self.compile_break_statement(label);
                Terminator::Break
            }
            Node::ContinueStatement { label } => {
                panic!("No implement"); // TODO
            }
            _ => never(),
        }
    }

    /// 编译函数声明，返回函数 value
    pub fn compile_function(
        &mut self,
        name: &str,
        args: &Vec<BasicMetadataTypeEnum<'ctx>>,
        arguments: &Vec<Box<Node>>,
        body: &Vec<Box<Node>>,
        return_kind: &Kind,
        is_only_block: bool,
    ) -> FunctionValue<'ctx> {
        let fn_value = self.build_fn_value(name, return_kind, args.as_slice());
        let entry_block = self.context.append_basic_block(fn_value, "entry");
        self.push_block_scope(entry_block); // 作用域入栈

        // 更新当前正在解析的函数
        self.current_fn = Some(fn_value);

        // 设置形参
        let mut arg_kind_names = vec![];
        for (i, arg) in fn_value.get_param_iter().enumerate() {
            let (arg_name, kind) = arguments[i].deref().read_identifier();
            arg_kind_names.push(*kind.read_kind_name().unwrap());
            let arg_value = fn_value.get_nth_param(i as u32).unwrap();
            self.put_variable(arg_name, *kind, Some(arg_value), true);

            // 为每个参数设置名称
            match kind.read_kind_name().unwrap() {
                KindName::Number => {
                    let fv = arg.into_float_value();
                    fv.set_name(arg_name);
                }
                KindName::Boolean => {
                    let fv = arg.into_int_value();
                    fv.set_name(arg_name);
                }
                KindName::Void => never(),
            }
        }
        if !is_only_block {
            self.put_function(name, &fn_value, arg_kind_names, &return_kind);
        }

        // 编译函数体
        let terminator = self.compile_block_statement(body, false);
        if !terminator.is_return() {
            self.builder.build_return(None);
        }

        self.pop_block_scope();

        // 验证函数，输出错误信息
        if !fn_value.verify(true) {
            if self.is_debug {
                println!("\n======================= IR =======================\n");
                self.module.print_to_stderr();
                println!("\n");
                panic!("Compile function failure");
            }
            unsafe { fn_value.delete() }
        }

        fn_value
    }

    // 编译块语句，返回在语句中间是否执行了 return 语句
    pub fn compile_block_statement(
        &mut self,
        statements: &Vec<Box<Node>>,
        is_new_scope: bool,
    ) -> Terminator {
        let mut after_block = None;
        if is_new_scope {
            let fn_value = self.current_fn.unwrap();
            let basic_block = self.context.append_basic_block(fn_value, "block");
            after_block = Some(self.context.append_basic_block(fn_value, "after_block"));
            self.builder.build_unconditional_branch(basic_block); // 切换到块

            // 作用域入栈
            self.push_block_scope(basic_block);
        }

        let mut terminator = Terminator::None;
        for stat in statements.iter() {
            terminator = self.compile_statement(stat.deref());
            if terminator.is_terminated() {
                break;
            }
        }

        if is_new_scope {
            let after_block = after_block.unwrap();
            if terminator.is_break() {
                self.builder.build_unconditional_branch(after_block); // 切换到块后续
            } else {
                unsafe {
                    after_block.delete();
                }
            }
            self.pop_block_scope();

            // 设置块后续为最后的位置，以便于继续编译下面的代码
            self.builder.position_at_end(after_block);
        }
        terminator
    }

    // 编译 if 语句
    pub fn compile_if_statement(
        &mut self,
        condition: &Node,
        consequent: &Node,
        alternate: &Option<Box<Node>>,
    ) -> Terminator {
        let condition_value = self.compile_expression(condition.deref()).into_int_value();
        let infer_condition_kind = self.infer_expression_kind(condition.deref());
        if *infer_condition_kind.read_kind_name().unwrap() != KindName::Boolean {
            panic!("If condition expression must be a boolean type");
        }

        let fn_value = self.current_fn.unwrap();
        let then_block = self.context.append_basic_block(fn_value, "then");
        let else_block = self.context.append_basic_block(fn_value, "else");
        let if_after_block = self.context.append_basic_block(fn_value, "if_after");

        // build condition branch
        self.builder
            .build_conditional_branch(condition_value, then_block, else_block);

        // build then block
        self.push_block_scope(then_block);
        let mut then_terminator =
            self.compile_block_statement(consequent.read_block_body(), false);
        if !then_terminator.is_return() {
            if then_terminator.is_break() {
                if let Some(v) = &self.labels.last_break_label {
                    self.builder.build_unconditional_branch(v.after_block);
                }
            } else {
                self.builder.build_unconditional_branch(if_after_block);
            }
        }
        self.pop_block_scope();

        // build else block
        let mut else_terminator = Terminator::None;
        self.push_block_scope(else_block);
        if alternate.is_some() {
            match alternate.as_ref().unwrap().deref() {
                // else-if
                Node::IfStatement {
                    condition,
                    consequent,
                    alternate,
                } => {
                    else_terminator = self.compile_if_statement(
                        condition.deref(),
                        consequent.deref(),
                        alternate,
                    );
                }
                // else
                Node::BlockStatement { body } => {
                    else_terminator = self.compile_block_statement(body, false);
                }
                _ => never(),
            }
        }
        if !else_terminator.is_terminated() {
            self.builder.build_unconditional_branch(if_after_block);
        }
        self.pop_block_scope();

        // 继续构建 if 语句之后的逻辑
        self.builder.position_at_end(if_after_block);

        // 如果 if / else 都return了，下方的代码直接不用执行了
        if then_terminator.is_terminated() && else_terminator.is_terminated() {
            unsafe {
                if_after_block.delete();
            }
            return then_terminator.merge(else_terminator);
        }
        Terminator::None
    }

    pub fn compile_loop_statement(
        &mut self,
        label: &Option<String>,
        body: &Node,
    ) -> Terminator {
        let label_name = match label {
            None => "",
            Some(v) => v,
        };
        let fn_value = self.current_fn.unwrap();
        let loop_block = self.context.append_basic_block(fn_value, "loop");
        let loop_then_block = self.context.append_basic_block(fn_value, "loop_then");
        let loop_after_block = self.context.append_basic_block(fn_value, "loop_after");

        // 切换到循环块判断条件
        let condition_ptr = self
            .builder
            .build_alloca(self.build_bool_type(), "loop_condition");
        self.builder
            .build_store(condition_ptr, self.build_bool_value(true));
        self.builder.build_unconditional_branch(loop_block);

        self.builder.position_at_end(loop_block);
        self.builder.build_conditional_branch(
            self.builder
                .build_load(condition_ptr, "do_loop")
                .into_int_value(),
            loop_then_block,
            loop_after_block,
        );

        // 块作用域入栈
        self.push_block_scope(loop_then_block);
        self.labels
            .push(label.clone(), condition_ptr, loop_block, loop_after_block);

        // 编译循环块
        let terminator = self.compile_block_statement(body.read_block_body(), false);

        if !terminator.is_terminated() {
            // 循环块结束后重新开始循环
            self.builder.build_unconditional_branch(loop_block);
        } else if terminator.is_break() {
            // 这里应该要跳转到 break label 的那个 after_block，不一定是当前的
            match &self.labels.last_break_label {
                Some(v) => {
                    self.builder.build_unconditional_branch(v.after_block);
                }
                None => {
                    self.builder.build_unconditional_branch(loop_after_block);
                }
            }
        } else {
            // 循环中间被 return
            // TODO: 这里强制给 loop_after 块加了跳转到自身，只是为了避免报错，后续再解决这个问题，这里永远不会执行
            self.builder.position_at_end(loop_after_block);
            self.builder.build_unconditional_branch(loop_after_block);
        }
        self.pop_block_scope();
        self.labels.pop();

        // 继续编译循环块下面的代码
        self.builder.position_at_end(loop_after_block);

        // 循环中只有 return 才透传结束者
        if terminator.is_return() {
            terminator
        } else {
            Terminator::None
        }
    }

    pub fn compile_return_statement(&mut self, argument: &Option<Box<Node>>) {
        match argument {
            Some(v) => {
                self.builder
                    .build_return(Some(&self.compile_expression(v.deref())));
            }
            None => {
                self.builder.build_return(None);
            }
        };
    }

    pub fn compile_break_statement(&mut self, label: &Option<String>) {
        match label {
            Some(label_mame) => {
                let label = self
                    .labels
                    .get(label_mame)
                    .expect(&format!("Label `{}` is not found", label_mame));
                self.labels.last_break_label = Some(label.clone());
                let parent_labels = self.labels.get_after_all(label_mame).unwrap();
                for item in parent_labels.iter() {
                    self.builder
                        .build_store(item.condition_ptr, self.build_bool_value(false));
                }
            }
            None => {
                let current = self.labels.current().unwrap();
                self.builder
                    .build_store(current.condition_ptr, self.build_bool_value(false));
                self.labels.last_break_label = Some(current.clone());
            }
        };
    }

    pub fn compile_variable_statement(&mut self, id: &Node, init: &Node) {
        let (id, mut kind) = id.read_identifier();

        // Note: 避免下面的临时变量生命周期不够长，临时借用变量
        let temp_borrowed;
        if !kind.is_exact() {
            temp_borrowed = self.infer_expression_kind(init);
            kind = &temp_borrowed;
        }
        let init_value = self.compile_expression(init);
        self.put_variable(id, *kind, Some(init_value.as_basic_value_enum()), false);
    }
}
