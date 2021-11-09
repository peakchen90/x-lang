use crate::helper::never;
use crate::scope::{BlockScope, ScopeType};
use inkwell::basic_block::BasicBlock;
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
use x_lang_ast::visitor::Visitor;

pub struct Compiler<'a, 'ctx> {
    pub context: &'ctx Context,
    pub builder: &'a Builder<'ctx>,
    pub module: &'a Module<'ctx>,
    pub scope: &'a mut BlockScope<'ctx>,
    pub execution_engine: &'a ExecutionEngine<'ctx>,
    pub print_fns: HashMap<&'a str, FunctionValue<'ctx>>,
    pub current_fn_value: Option<FunctionValue<'ctx>>,
    bootstrap_fn: Option<FunctionValue<'ctx>>,
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
            bootstrap_fn: None,
            current_fn_value: None,
            print_fns: HashMap::new(),
        };

        // 开始编译
        compiler.compile_program(ast);

        if is_debug {
            // 控制台打印 IR 码
            println!("\n================================ LLVM-IR ================================");
            module.print_to_stderr();

            println!("\n================================ OUTPUT =================================");
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

        unsafe {
            execution_engine.run_function(compiler.bootstrap_fn.unwrap(), &vec![]);
        }
    }

    pub fn put_variable(
        &mut self,
        name: &str,
        kind: Kind,
        value: Option<BasicValueEnum<'ctx>>,
        is_arg: bool, // 是否是函数参数
    ) {
        let current = self.scope.current().unwrap();
        if current.has(name) {
            panic!("Scope name `{}` is exist", name);
        }

        // 内存中的描述名称
        let mem_name = match is_arg {
            true => format!("ARGUMENT.{}", name),
            false => name.to_string(),
        };
        let ptr = match kind
            .read_kind_name()
            .expect("Can not declare void type variable")
        {
            KindName::Number => {
                let ty = self.build_number_type();
                let ptr = self.builder.build_alloca(ty, &mem_name);
                if let Some(v) = value {
                    self.builder.build_store(ptr, v.into_float_value());
                }
                Some(ptr)
            }
            KindName::Boolean => {
                let ty = self.build_bool_type();
                let ptr = self.builder.build_alloca(ty, &mem_name);
                if let Some(v) = value {
                    self.builder.build_store(ptr, v.into_int_value());
                }
                Some(ptr)
            }
            KindName::Void => None,
        };

        let scope_type = ScopeType::Variable { kind, ptr };
        self.scope.put_variable(name, scope_type);
    }

    pub fn put_function(
        &mut self,
        name: &str,
        fn_value: &FunctionValue<'ctx>,
        arg_kind_names: Vec<KindName>,
        return_kind: &Kind,
    ) {
        let mut root = self.scope.root().unwrap();
        if root.has(name) {
            panic!("Scope name `{}` is exist", name);
        }

        let scope_type = ScopeType::Function {
            fn_value: *fn_value,
            return_kind: *return_kind,
            arg_kind_names,
        };
        root.add(name, scope_type);
    }

    pub fn push_block_scope(&mut self, basic_block: BasicBlock<'ctx>) {
        self.scope.push(basic_block);
        self.builder.position_at_end(basic_block);
    }

    pub fn pop_block_scope(&mut self) {
        self.scope.pop();
        if let Some(v) = self.scope.current() {
            if let Some(b) = v.basic_block {
                self.builder.position_at_end(b);
            }
        }
    }

    // 推断表达式的返回类型
    pub fn infer_expression_kind(&self, expr: &Node) -> Kind {
        let mut ret_kind = Kind::None;

        Visitor::walk(expr, &mut |node, mut visitor| match node {
            Node::CallExpression { callee, .. } => {
                let (name, ..) = callee.deref().read_identifier();
                match self.scope.search_by_name(name, false) {
                    Some(v) => {
                        if v.is_fn() {
                            let (.., return_kind) = v.get_fn();
                            ret_kind = *return_kind;
                            visitor.stop();
                        }
                    }
                    _ => panic!("Function `{}` is not found", name),
                }
            }
            Node::BinaryExpression { left, .. } => {
                ret_kind = self.infer_expression_kind(left.deref());
                visitor.stop();
            }
            Node::AssignmentExpression { left, .. } => {
                ret_kind = self.infer_expression_kind(left.deref());
                visitor.stop();
            }
            Node::Identifier { name, kind } => match kind {
                Kind::Some(_) => {
                    ret_kind = *kind;
                    visitor.stop();
                }
                Kind::Infer => match self.scope.search_by_name(name, false) {
                    Some(v) => {
                        if v.is_var() {
                            let (kind, ..) = v.get_var();
                            if kind.is_exact() {
                                ret_kind = *kind;
                                visitor.stop();
                            }
                        }
                    }
                    _ => {}
                },
                _ => {}
            },
            Node::NumberLiteral { .. } => {
                ret_kind = Kind::create("num");
                visitor.stop();
            }
            Node::BooleanLiteral { .. } => {
                ret_kind = Kind::create("bool");
                visitor.stop();
            }
            _ => never(),
        });
        ret_kind
    }

    pub fn compile_program(&mut self, node: &Node) {
        self.inject_build_in();

        match node {
            Node::Program { body } => {
                self.bootstrap_fn = Some(self.compile_function(
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

    pub fn compile_statement(&mut self, node: &Node) {
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
            }
            Node::VariableDeclaration { id, init } => {
                let (id, mut kind) = id.deref().read_identifier();
                let init = init.deref();

                // Note: 避免下面的临时变量生命周期不够长，临时借用变量
                let temp_borrowed;
                if !kind.is_exact() {
                    temp_borrowed = self.infer_expression_kind(init);
                    kind = &temp_borrowed;
                }
                let init_value = self.compile_expression(init);
                self.put_variable(
                    id,
                    *kind,
                    Some(init_value.as_basic_value_enum()),
                    false,
                );
            }
            Node::BlockStatement { body } => {
                let fn_value = self.current_fn_value.unwrap();
                let basic_block = self.context.append_basic_block(fn_value, "block");
                let continue_block =
                    self.context.append_basic_block(fn_value, "block_continue");
                self.builder.build_unconditional_branch(basic_block); // 切换到块

                // 作用域入栈
                self.push_block_scope(basic_block);
                self.compile_block_statement(body);
                self.builder.build_unconditional_branch(continue_block); // 切换到块继续
                self.pop_block_scope();

                // 设置块继续为最后的位置，以便于继续编译下面的代码
                self.builder.position_at_end(continue_block);
            }
            Node::ReturnStatement { argument } => match argument {
                Some(v) => {
                    self.builder
                        .build_return(Some(&self.compile_expression(v.deref())));
                }
                None => {
                    self.builder.build_return(None);
                }
            },
            Node::ExpressionStatement { expression } => {
                self.compile_expression(expression.deref());
            }
            Node::IfStatement {
                condition,
                consequent,
                alternate,
            } => {
                self.compile_if_statement(condition, consequent, alternate);
            }
            _ => never(),
        }
    }

    /// 编译函数声明
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
        let block = self.context.append_basic_block(fn_value, "entry");
        self.push_block_scope(block); // 作用域入栈

        // 更新当前正在解析的函数
        self.current_fn_value = Some(fn_value);

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

        // compile function body
        for stat in body.iter() {
            self.compile_statement(stat.deref());
        }
        // 函数体最后都返回 void 作为返回的默认值
        self.builder.build_return(None);

        self.pop_block_scope();

        fn_value
    }

    // 编译块语句
    pub fn compile_block_statement(&mut self, statements: &Vec<Box<Node>>) {
        for stat in statements.iter() {
            self.compile_statement(stat.deref());
        }
    }

    // 编译 if 语句
    pub fn compile_if_statement(
        &mut self,
        condition: &Node,
        consequent: &Node,
        alternate: &Option<Box<Node>>,
    ) {
        let infer_condition_kind = self.infer_expression_kind(condition.deref());
        if *infer_condition_kind.read_kind_name().unwrap() != KindName::Boolean {
            panic!("If condition expression must be a boolean type");
        }

        let fn_value = self.current_fn_value.unwrap();
        let then_block = self.context.append_basic_block(fn_value, "then");
        let else_block = self.context.append_basic_block(fn_value, "else");
        let if_continue_block = self.context.append_basic_block(fn_value, "if_continue");

        // build condition branch
        let condition = self.compile_expression(condition.deref()).into_int_value();
        self.builder
            .build_conditional_branch(condition, then_block, else_block);

        // build then block
        self.push_block_scope(then_block);
        self.compile_block_statement(consequent.read_block_body());
        self.builder.build_unconditional_branch(if_continue_block);
        self.pop_block_scope();

        // build else block
        self.push_block_scope(else_block);
        if alternate.is_some() {
            match alternate.as_ref().unwrap().deref() {
                // else-if
                Node::IfStatement {
                    condition,
                    consequent,
                    alternate,
                } => {
                    self.compile_if_statement(
                        condition.deref(),
                        consequent.deref(),
                        alternate,
                    );
                }
                // else
                Node::BlockStatement { body } => {
                    self.compile_block_statement(body);
                }
                _ => never(),
            }
        }
        self.builder.build_unconditional_branch(if_continue_block);
        self.pop_block_scope();

        // 继续构建 if 语句之后的逻辑
        self.builder.position_at_end(if_continue_block);

        // let phi = self.builder.build_phi(self.build_number_type(), "iftmp");
        // phi.add_incoming(&[
        //     (&(self.build_number_value(1.0).into()), then_block),
        //     (&(self.build_number_value(1.0).into()), else_block),
        // ]);
    }
}
