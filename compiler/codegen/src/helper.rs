use crate::compiler::Compiler;
use crate::scope::ScopeType;
use inkwell::comdat::ComdatSelectionKind;
use inkwell::context::Context;
use inkwell::types::{FloatType, VoidType};
use inkwell::values::{
    BasicValue, BasicValueEnum, FloatValue, FunctionValue, PointerValue,
};
use llvm_sys::comdat::LLVMGetOrInsertComdat;
use llvm_sys::core::LLVMGetGlobalContext;
use x_lang_ast::shared::{Kind, KindName};

// 永从不会发生，用于避免编译器报错
pub fn never() -> ! {
    panic!("NEVER")
}

// build-in print impl
extern "C" fn build_in_print(n: f64) {
    println!("{}", n);
}

impl<'a, 'ctx> Compiler<'a, 'ctx> {
    pub fn build_number_type(&self) -> FloatType<'ctx> {
        self.context.f64_type()
    }

    pub fn build_number_value(&self, value: f64) -> FloatValue<'ctx> {
        self.build_number_type().const_float(value)
    }

    pub fn build_void_type(&self) -> VoidType<'ctx> {
        self.context.void_type()
    }

    pub fn get_declare_var(
        &self,
        name: &str,
    ) -> (&Kind, &Option<PointerValue<'ctx>>) {
        self.scope
            .search_by_name(name)
            .expect(&format!("Variable `{}` is not found", name))
            .get_var()
    }

    pub fn get_declare_var_ptr(
        &self,
        name: &str,
    ) -> &Option<PointerValue<'ctx>> {
        let (_, ptr) = self.get_declare_var(name);
        ptr
    }

    pub fn get_declare_fn(
        &self,
        name: &str,
    ) -> (&FunctionValue<'ctx>, &Vec<KindName>, &Kind) {
        self.scope
            .search_by_name(name)
            .expect(&format!("Function `{}` is not declare", name))
            .get_fn()
    }

    pub fn get_declare_fn_val(&self, name: &str) -> &FunctionValue<'ctx> {
        let (fn_value, ..) = self.get_declare_fn(name);
        fn_value
    }

    // built-in
    pub fn inject_build_in(&mut self) {
        self.scope.push_without_block();

        unsafe {
            let ext_fn_type = self
                .build_void_type()
                .fn_type(&[self.build_number_type().into()], false);
            let ext_fn_value =
                self.module.add_function("print", ext_fn_type, None);
            self.execution_engine
                .add_global_mapping(&ext_fn_value, build_in_print as usize);

            // insert scope
            self.scope.put_variable(
                "print",
                ScopeType::Function {
                    return_kind: Kind::Some(KindName::Void),
                    arg_kind_names: vec![KindName::Number],
                    fn_value: ext_fn_value,
                },
            );
        }
    }
}
