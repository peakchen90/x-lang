use crate::scope::{BlockScope, KindValue, ScopeDecl, VariableDecl};
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::ExecutionEngine;
use inkwell::module::Module;
use inkwell::values::FloatValue;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use x_lang_ast::shared::{Kind, KindName, Node};
use x_lang_ast::visit::Visitor;

pub struct Compiler<'a, 'ctx> {
    pub context: &'ctx Context,
    pub builder: &'a Builder<'ctx>,
    pub module: &'a Module<'ctx>,
    pub scope: &'a mut BlockScope<'ctx>,
    // pub execution_engine: ExecutionEngine<'ctx>,
}

impl<'a, 'ctx> Compiler<'a, 'ctx> {
    pub fn compile(ast: &Node) {
        let context = &Context::create();
        let module = &context.create_module("main");
        let builder = &context.create_builder();
        let scope = &mut BlockScope::new();

        let compiler = Compiler {
            context,
            module,
            builder,
            scope,
        };

        compiler.compile_program(ast);
    }

    fn create_block_stack_alloc(&self, name: &str) {
        let builder = self.context.create_builder();
        // builder.
        // self.builder.build_alloca(name, "");
    }

    fn pub_variable(&mut self, name: &str, kind: Kind, value: KindValue<'ctx>) {
        let mut current = self.scope.current().unwrap();
        if current.has(name) {
            panic!("Scope name `{}` is exist", name);
        }

        let decl = VariableDecl { kind, value };
        self.scope.put_variable(name, decl);

        match kind.read_kind_name().unwrap() {
            KindName::Number => {
                let ft = self.context.f64_type();
                let ptr = self.builder.build_alloca(ft, name);
                self.builder.build_store(ptr, *value.read_number());
            }
        };
    }

    // 推断表达式的返回类型名称
    fn infer_expression_kind(&self, expr: &Node) -> Kind {
        let mut ret_kind = Kind::None;
        let mut x = RefCell::new(ret_kind);

        Visitor::walk(expr, &mut |node, mut visitor| match node {
            Node::CallExpression { ref callee, .. } => {
                let (id, ..) = callee.deref().read_identifier();
                match self.scope.search_variable(id) {
                    Some(v) => {
                        if v.is_fn() {
                            ret_kind = *&v.get_fn_decl().return_kind;
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
                                ret_kind = v.get_var_decl().kind;
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

    fn compile_program(&self, ast: &Node) {
        match ast {
            Node::Program { ref body } => {
                // let module = self.module("TODO");
                // module.add
                // for stat in body.iter() {
                //     self.compile_node(stat.deref());
                // }
                // self.context.f64_type().const_float(*value);
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
            } => {}
            Node::VariableDeclaration { ref id, ref init } => {
                let (id, mut kind) = id.deref().read_identifier();
                let init = init.deref();

                // Note: 避免下面的临时变量生命周期不够长
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
            }
            Node::ReturnStatement { ref argument } => {}
            Node::ExpressionStatement { .. } => {}
            _ => {}
        }
    }

    fn compile_expression(&self, ast: &Node) -> KindValue<'ctx> {
        match ast {
            // Node::CallExpression { .. } => {
            //
            // }
            // Node::BinaryExpression { .. } => {}
            // Node::AssignmentExpression { .. } => {}
            // Node::Identifier { .. } => {}
            Node::NumberLiteral { ref value } => {
                KindValue::Number(self.context.f64_type().const_float(*value))
            }
            _ => panic!("Error"),
        }
    }
}
