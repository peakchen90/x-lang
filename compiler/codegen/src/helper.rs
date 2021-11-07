use crate::compiler::Compiler;
use crate::scope::ScopeType;
use inkwell::types::{FloatType, VoidType};
use inkwell::values::{
    BasicValueEnum, FloatValue, FunctionValue, PointerValue,
};
use x_lang_ast::shared::Kind;

// 永从不会发生，用于避免编译器报错
pub fn never() -> ! {
    panic!("NEVER")
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

    pub fn get_declare_variable_ptr(
        &self,
        name: &str,
    ) -> &Option<PointerValue<'ctx>> {
        let (.., ptr) = self
            .scope
            .search_by_name(name)
            .expect(&format!("Variable `{}` is not found", name))
            .get_var();
        ptr
    }

    pub fn get_declare_fn(
        &self,
        name: &str,
    ) -> (&Kind, &FunctionValue<'ctx>, &PointerValue<'ctx>) {
        self.scope
            .search_by_name(name)
            .expect(&format!("Function `{}` is not declare", name))
            .get_fn()
    }

    pub fn get_declare_fn_val(&self, name: &str) -> &FunctionValue<'ctx> {
        let (_, fn_val, _) = self.get_declare_fn(name);
        fn_val
    }
}
