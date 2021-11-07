use crate::helper::LLVMDataValue;
use crate::helper::LLVMDataValue::Void;
use crate::scope::{BlockScope, ScopeType};
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::ExecutionEngine;
use inkwell::module::Module;
use inkwell::types::BasicMetadataTypeEnum;
use inkwell::values::{BasicValue, FunctionValue, PointerValue};
use std::ops::Deref;
use x_lang_ast::shared::{Kind, KindName, Node};
use x_lang_ast::visit::Visitor;

pub struct Compiler<'a, 'ctx> {
    pub context: &'ctx Context,
    pub builder: &'a Builder<'ctx>,
    pub module: &'a Module<'ctx>,
    pub scope: &'a mut BlockScope<'ctx>,
    // pub execution_engine: ExecutionEngine<'ctx>,

    current_fn_val: Option<FunctionValue<'ctx>>
}

impl<'a, 'ctx> Compiler<'a, 'ctx> {
    pub fn compile(ast: &Node) {
        let context = &Context::create();
        let module = &context.create_module("main");
        let builder = &context.create_builder();
        let scope = &mut BlockScope::new();

        let mut compiler = Compiler {
            context,
            module,
            builder,
            scope,
            current_fn_val: None
        };

        compiler.compile_program(ast);
    }

    fn create_block_stack_alloc(&self, name: &str) {
        // let builder = self.context.create_builder();
        // builder.
        // self.builder.build_alloca(name, "");
    }

    fn pub_variable(
        &mut self,
        name: &str,
        kind: Kind,
        value: LLVMDataValue<'ctx>,
    ) {
        let mut current = self.scope.current().unwrap();
        if current.has(name) {
            panic!("Scope name `{}` is exist", name);
        }

        let builder = self.builder;
        // let builder = self.context.create_builder();
        // builder.position_at_end(entry)

        // let entry = self.current_fn_val.unwrap().get_first_basic_block().unwrap();
        // match entry.get_first_instruction() {
        //     Some(first_instr) => builder.position_before(&first_instr),
        //     None => builder.position_at_end(entry)
        // }

        match kind.read_kind_name().unwrap() {
            KindName::Number => {
                let ft = self.context.f64_type();
                let ptr = builder.build_alloca(ft, name);
                self.builder.build_store(ptr, *value.read_number());
                let scope_type = ScopeType::Variable { kind, ptr };
                self.scope.put_variable(name, scope_type);
            }
            KindName::Void => {}
        };
    }

    // 推断表达式的返回类型名称
    fn infer_expression_kind(&self, expr: &Node) -> Kind {
        let mut ret_kind = Kind::None;

        Visitor::walk(expr, &mut |node, mut visitor| match node {
            Node::CallExpression { ref callee, .. } => {
                let (id, ..) = callee.deref().read_identifier();
                match self.scope.search_variable(id) {
                    Some(v) => {
                        if v.is_fn() {
                            let (return_kind, ..) = v.get_fn();
                            ret_kind = *return_kind;
                            visitor.stop();
                        }
                    }
                    _ => {}
                }
            }
            Node::BinaryExpression { ref left, .. } => {
                ret_kind = self.infer_expression_kind(left.deref());
                visitor.stop();
            }
            Node::AssignmentExpression { ref left, .. } => {
                ret_kind = self.infer_expression_kind(left.deref());
                visitor.stop();
            }
            Node::Identifier { ref name, ref kind } => match kind {
                Kind::Some(_) => {
                    ret_kind = *kind;
                    visitor.stop();
                }
                Kind::Infer => match self.scope.search_variable(name) {
                    Some(v) => {
                        if v.is_var() {
                            if kind.is_exact() {
                                let (kind, ..) = v.get_fn();
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
                ret_kind = Kind::Some(KindName::Number);
                visitor.stop();
            }
            _ => {}
        });
        ret_kind
    }

    fn compile_program(&mut self, ast: &Node) {
        match ast {
            Node::Program { ref body } => {
                self.scope.push();
                for stat in body.iter() {
                    self.compile_statement(stat.deref());
                }
                self.scope.pop();
            }
            _ => {}
        }
    }

    fn compile_statement(&mut self, node: &Node) {
        match node {
            Node::FunctionDeclaration {
                ref id,
                ref arguments,
                ref body,
                ref return_kind,
            } => {
                self.scope.push();

                let mut arg_names = vec![];
                let arg_types = arguments
                    .iter()
                    .map(|arg| {
                        let (name, kind) = arg.deref().read_identifier();
                        let kind_name = kind.read_kind_name().unwrap();

                        if let KindName::Number = kind_name {
                            arg_names.push((name, kind));
                            self.context.f64_type().into()
                        } else {
                            panic!("Error")
                        }
                    })
                    .collect::<Vec<BasicMetadataTypeEnum>>();

                let arg_types = arg_types.as_slice();
                let fn_type = self.context.f64_type().fn_type(arg_types, false);
                let (id, ..) = &id.deref().read_identifier();
                let fn_val = self.module.add_function(id, fn_type, None);

                self.module.print_to_string();

                // 为每个参数设置名称
                for (i, arg) in fn_val.get_param_iter().enumerate() {
                    let (arg_name, kind) = arg_names[i];
                    match kind.read_kind_name().unwrap() {
                        KindName::Number => {
                            let fv = arg.into_float_value();
                            fv.set_name(arg_name);
                            self.pub_variable(
                                arg_name,
                                *kind,
                                LLVMDataValue::Number(fv),
                            );
                        }
                        KindName::Void => {}
                    }
                }

                let block = self.context.append_basic_block(fn_val, "function");
                self.builder.position_at_end(block);

                // TODO update field
                self.current_fn_val = Some(fn_val);

                // compile function body
                self.compile_statement(body.deref());

                self.scope.pop();
            }
            Node::VariableDeclaration { ref id, ref init } => {
                let (id, mut kind) = id.deref().read_identifier();
                let init = init.deref();

                // Note: 避免下面的临时变量生命周期不够长，临时借用变量
                let temp_borrowed;
                if !kind.is_exact() {
                    temp_borrowed = self.infer_expression_kind(init);
                    kind = &temp_borrowed;
                }
                let init_value = self.compile_expression(init);
                self.pub_variable(id, *kind, init_value);
            }
            Node::BlockStatement { ref body } => {
                self.scope.push();
                for stat in body {
                    self.compile_statement(stat.deref());
                }
                self.scope.pop();
            }
            Node::ReturnStatement { ref argument } => {}
            Node::ExpressionStatement { .. } => {}
            _ => {}
        }
    }

    fn compile_expression(&self, ast: &Node) -> LLVMDataValue<'ctx> {
        match ast {
            // Node::CallExpression { .. } => {
            //
            // }
            // Node::BinaryExpression { .. } => {}
            // Node::AssignmentExpression { .. } => {}
            // Node::Identifier { .. } => {}
            Node::NumberLiteral { ref value } => LLVMDataValue::Number(
                self.context.f64_type().const_float(*value),
            ),
            _ => panic!("Error"),
        }
    }
}
