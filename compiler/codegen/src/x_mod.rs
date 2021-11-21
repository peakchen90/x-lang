//! TODO: 暂时不可用

use crate::build_in::system_print_num;
use crate::scope::{BlockScope, Scope};
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::BasicType;
use inkwell::OptimizationLevel;
use std::collections::HashMap;

pub struct XMod<'ctx> {
    public: Scope<'ctx>,
}

pub struct XModManager<'ctx> {
    pub mods: HashMap<String, XMod<'ctx>>,
    pub main_mod: XMod<'ctx>,
}

impl<'ctx> XMod<'ctx> {

}

/// test
pub fn x_mod() {
    let context = &Context::create();
    let module = context.create_module("main");
    let sub_mod = context.create_module("sub");
    let builder = context.create_builder();

    let exec = module
        .create_jit_execution_engine(OptimizationLevel::None)
        .unwrap();
    let sub_exec = sub_mod
        .create_jit_execution_engine(OptimizationLevel::None)
        .unwrap();
    // execution_engine.add_module(&sub_mod);

    // print
    let print_fn = sub_mod.add_function(
        "print",
        context
            .void_type()
            .fn_type(&[context.i32_type().into()], false),
        None,
    );
    sub_exec.add_global_mapping(&print_fn, system_print_num as usize);

    // sub
    let sub_fn_type = context.void_type().fn_type(&[], false);
    let sub_fn = sub_mod.add_function("abc", sub_fn_type, None);
    let b2 = context.append_basic_block(sub_fn, "entry");
    builder.position_at_end(b2);
    builder.build_call(
        print_fn,
        &[context.i32_type().const_int(666, false).into()],
        "call_print",
    );
    builder.build_return(None);
    if !sub_fn.verify(true) {
        println!("\n");
        sub_mod.print_to_stderr();
        panic!("ERROR")
    }

    // exec.add_module(&sub_mod);
    println!("----------------------------------------------------------------------------------");

    // main
    let main_fn_type = context.void_type().fn_type(&[], false);
    let main_fn = module.add_function("main", main_fn_type, None);
    let b1 = context.append_basic_block(main_fn, "entry");
    builder.position_at_end(b1);
    // builder.build_call(sub_fn, &[], "call_abc");

    // bind
    let sub_abc_fn =
        module.add_function("sub_abc", context.void_type().fn_type(&[], false), None);
    let sub_abc_ptr = sub_exec.get_function_address("abc").unwrap();
    exec.add_global_mapping(&sub_abc_fn, sub_abc_ptr);

    builder.build_call(sub_abc_fn, &[], "call_abc");
    builder.build_return(None);
    if !main_fn.verify(true) {
        println!("\n");
        module.print_to_stderr();
        panic!("ERROR")
    }

    unsafe {
        // 读取 main 函数并调用
        type MainFunction = unsafe extern "C" fn() -> isize;
        let main_fn = exec.get_function::<MainFunction>("main");
        if let Ok(main_fn) = main_fn {
            main_fn.call();
        }
    }

    // execution_engine.add_module(&sub_mod);
}
