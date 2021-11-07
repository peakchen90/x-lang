use crate::helper::to_char_ptr;
use crate::scope::{BlockScope, ScopeType};
use llvm_sys::core::{LLVMBuildAlloca, LLVMBuildStore, LLVMBuildVAArg, LLVMConstInt, LLVMContextCreate, LLVMCreateBuilderInContext, LLVMFloatType, LLVMInt64TypeInContext, LLVMModuleCreateWithNameInContext};
use llvm_sys::prelude::{
    LLVMBuilderRef, LLVMContextRef, LLVMModuleRef, LLVMValueRef,
};
use llvm_sys::LLVMValue;
use std::ops::Deref;
use x_lang_ast::shared::{Kind, KindName, Node};
use x_lang_ast::visit::Visitor;

pub struct Compiler<'ctx> {
    pub context: &'ctx LLVMContextRef,
    pub builder: &'ctx LLVMBuilderRef,
    pub module: &'ctx LLVMModuleRef,
    pub scope: &'ctx mut BlockScope,
    // pub execution_engine: ExecutionEngine<'ctx>,

    // current_fn_val: Option<FunctionValue<'ctx>>
}

impl<'ctx> Compiler<'ctx> {
    pub fn compile(ast: &Node) {
        unsafe {
            let context = &LLVMContextCreate();
            let module = &LLVMModuleCreateWithNameInContext(
                "main".as_ptr() as *const _,
                *context,
            );
            let builder = &LLVMCreateBuilderInContext(*context);

            let scope = &mut BlockScope::new();

            let mut compiler = Compiler {
                context,
                module,
                builder,
                scope,
                // current_fn_val: None
            };

            compiler.compile_program(ast);
        }
    }

    unsafe fn put_scope(
        &mut self,
        name: &str,
        kind: Kind,
        value: LLVMValueRef,
    ) {
        let mut current = self.scope.current().unwrap();
        if current.has(name) {
            panic!("Scope name `{}` is exist", name);
        }

        let builder = self.builder;

        match kind.read_kind_name().unwrap() {
            KindName::Number => {
                let ft = LLVMFloatType();
                let ptr = LLVMBuildAlloca(*self.builder, ft, to_char_ptr(name));
                let scope_type = ScopeType::Variable { kind, ptr };
                LLVMBuildStore(*self.builder, value, ptr);
                self.scope.put_variable(name, scope_type);
            }
            KindName::Void => {}
        };
    }

    unsafe fn compile_program(&mut self, ast: &Node) {
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

    unsafe fn compile_statement(&mut self, node: &Node) {
        match node {
            /*Node::FunctionDeclaration {
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
                            self.put_scope(
                                arg_name,
                                *kind,
                                3
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
            }*/
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
                self.put_scope(id, *kind, init_value);
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
}
