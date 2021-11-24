//! build-in impl
use crate::helper::never;
use crate::scope::{FunctionScope, ScopeType};
use crate::Compiler;
use inkwell::types::BasicMetadataTypeEnum;
use inkwell::values::{BasicValue, BasicValueEnum};
use inkwell::AddressSpace;
use std::ops::{Deref, Index};
use x_lang_ast::node::Node;
use x_lang_ast::shared::{Kind, KindName};

pub extern "C" fn system_print_newline() {
    print!("\n");
}

pub extern "C" fn system_print_str(value: &[u16]) {
    // for i in 0..50 {
    //     print!("{} ", value[i]);
    // }
    // println!();

    let mut size = 0;
    loop {
        if value[size] == 0 {
            break;
        }
        size += 1;
    }
    let str = String::from_utf16(&value[0..size]).expect("Invalid UTF-16 string");
    print!("{}", str);
}

pub extern "C" fn system_print_num(value: f64) {
    print!("{}", value);
}

pub extern "C" fn system_print_u64(value: u64) {
    print!("{}", value);
}

pub extern "C" fn system_print_bool(value: u8) {
    print!(
        "{}",
        match value {
            1 => "true",
            0 => "false",
            _ => never(),
        }
    );
}

impl<'ctx> Compiler<'ctx> {
    // built-in
    pub fn inject_build_in(&mut self) {
        unsafe {
            // print string
            let ptr_type = self.context.i16_type().ptr_type(AddressSpace::Generic);
            self.bind_system_print_fn(
                "str",
                &[ptr_type.into()],
                system_print_str as usize,
            );
            // print number
            self.bind_system_print_fn(
                "num",
                &[self.build_number_type().into()],
                system_print_num as usize,
            );
            // print i64 number
            self.bind_system_print_fn(
                "u64",
                &[self.context.i64_type().into()],
                system_print_u64 as usize,
            );
            // print bool
            self.bind_system_print_fn(
                "bool",
                &[self.build_bool_type().into()],
                system_print_bool as usize,
            );
            // print newline
            self.bind_system_print_fn("newline", &[], system_print_newline as usize);
        }
    }

    fn bind_system_print_fn(
        &mut self,
        type_name: &'static str,
        arg_types: &[BasicMetadataTypeEnum<'ctx>],
        address: usize,
    ) {
        let print_fn_value = self.build_fn_value(
            &format!("print_{}", type_name),
            &Kind::create("void"),
            arg_types,
        );
        self.execution_engine
            .add_global_mapping(&print_fn_value, address);

        // 保存信息（接收任意类型，不校验参数）
        self.print_fns.insert(type_name, print_fn_value);
        self.scope.external.add(
            "print",
            ScopeType::Function(FunctionScope {
                fn_value: print_fn_value,
                return_kind: Kind::create("void"),
                arg_kind_names: vec![],
                arg_variables: vec![],
                entry_block: None,
            }),
        )
    }

    pub fn build_call_system_print(
        &self,
        arguments: &Vec<Box<Node>>,
    ) -> BasicValueEnum<'ctx> {
        for (i, arg) in arguments.iter().enumerate() {
            let arg = arg.deref();
            let arg_value = self.compile_expression(arg);
            let infer_kind = self.infer_expression_kind(arg);
            let infer_kind_name = infer_kind.read_kind_name().unwrap();
            match infer_kind_name {
                KindName::Number | KindName::Boolean => {
                    let fn_value = self
                        .print_fns
                        .get(infer_kind_name.to_string().as_str())
                        .unwrap();
                    self.build_call_fn(fn_value, &[arg_value.into()], "sys_print");
                }
                KindName::String => {
                    let fn_value = self.print_fns.get("str").unwrap();
                    let address = arg_value.into_int_value();
                    let ptr = self.build_cast_string_address(address);
                    let ptr = self.build_read_string_ptr(ptr);
                    self.build_call_fn(fn_value, &[ptr.into()], "sys_print");
                }
                KindName::Void => {}
            }
        }
        // 打印换行
        self.build_call_fn(
            self.print_fns.get("newline").unwrap(),
            &[],
            "sys_print_newline",
        );

        self.build_null_value().as_basic_value_enum()
    }
}
