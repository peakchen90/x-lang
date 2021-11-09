use crate::helper::never;
use crate::scope::BlockScope;
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
use std::path::Path;
use x_lang_ast::node::Node;
use x_lang_ast::shared::{Kind, KindName};

pub struct Compiler<'a, 'ctx> {
    pub context: &'ctx Context,
    pub builder: &'a Builder<'ctx>,
    pub module: &'a Module<'ctx>,
    pub scope: &'a mut BlockScope<'ctx>,
    pub execution_engine: &'a ExecutionEngine<'ctx>,
    pub print_fns: HashMap<&'a str, FunctionValue<'ctx>>,
    pub main_fn: Option<FunctionValue<'ctx>>,
    pub current_fn: Option<FunctionValue<'ctx>>,
    pub is_debug: bool,
}

impl<'a, 'ctx> Compiler<'a, 'ctx> {
    pub fn compile(ast: &Node, is_debug: bool) {
        let context = &Context::create();
        let module = &context.create_module("main");
        let builder = &context.create_builder();
        let scope = &mut BlockScope::new();
        let execution_engine = &module
            .create_jit_execution_engine(OptimizationLevel::None)
            .unwrap();

        let mut compiler = Compiler {
            context,
            module,
            builder,
            scope,
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
            // 控制台打印 IR 码
            println!("\n================================ LLVM-IR ================================");
            module.print_to_stderr();

            println!("\n================================ OUTPUT =================================");
        }

        unsafe {
            execution_engine.run_function(compiler.main_fn.unwrap(), &vec![]);
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

    // 编译一条语句，返回在语句中间是否执行了 return 语句
    pub fn compile_statement(&mut self, node: &Node) -> bool {
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
                false
            }
            Node::VariableDeclaration { id, init } => {
                self.compile_variable_statement(id.deref(), init.deref());
                false
            }
            Node::BlockStatement { body } => self.compile_block_statement(body, true),
            Node::ReturnStatement { argument } => {
                self.compile_return_statement(argument);
                true
            }
            Node::ExpressionStatement { expression } => {
                self.compile_expression(expression.deref());
                false
            }
            Node::IfStatement {
                condition,
                consequent,
                alternate,
            } => self.compile_if_statement(condition, consequent, alternate),
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
        let is_returned = self.compile_block_statement(body, false);
        if !is_returned {
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
    ) -> bool {
        let mut continue_block = None;
        if is_new_scope {
            let fn_value = self.current_fn.unwrap();
            let basic_block = self.context.append_basic_block(fn_value, "block");
            continue_block =
                Some(self.context.append_basic_block(fn_value, "block_continue"));
            self.builder.build_unconditional_branch(basic_block); // 切换到块

            // 作用域入栈
            self.push_block_scope(basic_block);
        }

        let mut is_returned = false;
        for stat in statements.iter() {
            is_returned = self.compile_statement(stat.deref());
            if is_returned {
                break;
            }
        }

        if is_new_scope {
            let continue_block = continue_block.unwrap();
            if !is_returned {
                self.builder.build_unconditional_branch(continue_block); // 切换到块后续
            } else {
                unsafe {
                    continue_block.delete();
                }
            }
            self.pop_block_scope();

            // 设置块后续为最后的位置，以便于继续编译下面的代码
            self.builder.position_at_end(continue_block);
        }
        is_returned
    }

    // 编译 if 语句
    pub fn compile_if_statement(
        &mut self,
        condition: &Node,
        consequent: &Node,
        alternate: &Option<Box<Node>>,
    ) -> bool {
        let condition_value = self.compile_expression(condition.deref()).into_int_value();
        let infer_condition_kind = self.infer_expression_kind(condition.deref());
        if *infer_condition_kind.read_kind_name().unwrap() != KindName::Boolean {
            panic!("If condition expression must be a boolean type");
        }

        let fn_value = self.current_fn.unwrap();
        let then_block = self.context.append_basic_block(fn_value, "then");
        let else_block = self.context.append_basic_block(fn_value, "else");
        let if_continue_block = self.context.append_basic_block(fn_value, "if_continue");

        // build condition branch
        self.builder
            .build_conditional_branch(condition_value, then_block, else_block);

        // build then block
        self.push_block_scope(then_block);
        let mut is_then_returned =
            self.compile_block_statement(consequent.read_block_body(), false);
        if !is_then_returned {
            self.builder.build_unconditional_branch(if_continue_block);
        }
        self.pop_block_scope();

        // build else block
        let mut is_else_returned = false;
        self.push_block_scope(else_block);
        if alternate.is_some() {
            match alternate.as_ref().unwrap().deref() {
                // else-if
                Node::IfStatement {
                    condition,
                    consequent,
                    alternate,
                } => {
                    is_else_returned = self.compile_if_statement(
                        condition.deref(),
                        consequent.deref(),
                        alternate,
                    );
                }
                // else
                Node::BlockStatement { body } => {
                    is_else_returned = self.compile_block_statement(body, false);
                }
                _ => never(),
            }
        }
        if !is_else_returned {
            self.builder.build_unconditional_branch(if_continue_block);
        }
        self.pop_block_scope();

        // 继续构建 if 语句之后的逻辑
        self.builder.position_at_end(if_continue_block);

        // 如果 if / else 都return了，下方的代码直接不用执行了
        if is_then_returned && is_else_returned {
            unsafe {
                if_continue_block.delete();
            }
            return true;
        }
        false
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
