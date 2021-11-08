use crate::build_in::{system_print_bool, system_print_newline, system_print_num, system_print_str};
use crate::compiler::Compiler;
use crate::scope::ScopeType;
use inkwell::comdat::ComdatSelectionKind;
use inkwell::context::Context;
use inkwell::types::{
    BasicMetadataTypeEnum, FloatType, FunctionType, IntType, VoidType,
};
use inkwell::values::{
    BasicMetadataValueEnum, BasicValue, BasicValueEnum, FloatValue,
    FunctionValue, IntValue, PointerValue,
};
use std::env::args;
use std::ops::Deref;
use x_lang_ast::shared::{Kind, KindName, Node};

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

    pub fn build_bool_type(&self) -> IntType<'ctx> {
        self.context.bool_type()
    }

    pub fn build_bool_value(&self, value: bool) -> IntValue<'ctx> {
        self.build_bool_type().const_int(
            match value {
                true => 1,
                false => 0,
            },
            false,
        )
    }

    pub fn build_fn_type(
        &self,
        return_kind: &Kind,
        args: &[BasicMetadataTypeEnum<'ctx>],
    ) -> FunctionType<'ctx> {
        match return_kind.read_kind_name().unwrap() {
            KindName::Number => self.build_number_type().fn_type(args, false),
            KindName::Boolean => self.build_bool_type().fn_type(args, false),
            KindName::Void => self.build_void_type().fn_type(args, false),
        }
    }

    pub fn build_fn_value(
        &self,
        name: &str,
        return_kind: &Kind,
        args: &[BasicMetadataTypeEnum<'ctx>],
    ) -> FunctionValue<'ctx> {
        let fn_type = self.build_fn_type(return_kind, args);
        self.module.add_function(name, fn_type, None)
    }

    pub fn build_void_type(&self) -> VoidType<'ctx> {
        self.context.void_type()
    }

    pub fn build_null_value(&self) -> IntValue<'ctx> {
        self.build_bool_type().const_zero()
    }

    pub fn build_call_fn(
        &self,
        fn_value: &FunctionValue<'ctx>,
        args: &[BasicMetadataValueEnum<'ctx>],
        name: &str,
    ) -> BasicValueEnum<'ctx> {
        match self
            .builder
            .build_call(*fn_value, args, &format!("CALL.{}", name))
            .try_as_basic_value()
            .left()
        {
            Some(v) => v,
            None => self.build_null_value().as_basic_value_enum(), // TODO: 调用失败返回 0 ??
        }
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
            // print string
            // self.bind_system_print_fn(
            //     "str",
            //     self.bui,
            //     system_print_str as usize,
            // );
            // print number
            self.bind_system_print_fn(
                "num",
                Some(self.build_number_type().into()),
                system_print_num as usize,
            );
            // print bool
            self.bind_system_print_fn(
                "bool",
                Some(self.build_bool_type().into()),
                system_print_bool as usize,
            );
            // print newline
            self.bind_system_print_fn(
                "newline",
                None,
                system_print_newline as usize,
            );
        }
    }

    fn bind_system_print_fn(
        &mut self,
        type_name: &'a str,
        arg_type: Option<BasicMetadataTypeEnum<'ctx>>,
        address: usize,
    ) {
        let arg_types = match arg_type {
            Some(v) => vec![v],
            None => vec![],
        };
        let print_fn_value = self.build_fn_value(
            "print",
            &Kind::create("void"),
            arg_types.as_slice(),
        );
        self.execution_engine
            .add_global_mapping(&print_fn_value, address);
        self.print_fns.insert(type_name, print_fn_value);
    }

    pub fn build_call_system_print(
        &self,
        arguments: &Vec<Box<Node>>,
    ) -> BasicValueEnum<'ctx> {
        let len = arguments.len();
        for (i, arg) in arguments.iter().enumerate() {
            let arg = arg.deref();
            let infer_kind = self.infer_expression_kind(arg);
            let infer_kind_name = infer_kind.read_kind_name().unwrap();
            match infer_kind_name {
                KindName::Number | KindName::Boolean => {
                    let fn_value = self
                        .print_fns
                        .get(infer_kind_name.to_string().as_str())
                        .unwrap();
                    let arg_value = self.compile_expression(arg);
                    self.build_call_fn(
                        fn_value,
                        &[arg_value.into()],
                        "CALL.sys_print",
                    );
                }
                KindName::Void => {}
            }
        }
        // 打印换行
        self.build_call_fn(
            self.print_fns.get("newline").unwrap(),
            &[],
            "CALL.sys_print_newline",
        );

        self.build_null_value().as_basic_value_enum()
    }
}
