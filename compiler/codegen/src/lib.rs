mod compiler;
mod scope;
mod helper;

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::{ExecutionEngine, JitFunction};
use inkwell::module::Module;
use inkwell::values::VectorValue;
use inkwell::OptimizationLevel;
use std::error::Error;

type SumFunc = unsafe extern "C" fn(i64, i64, i64) -> i64;

pub struct CodeGen__<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    execution_engine: ExecutionEngine<'ctx>,
}

impl<'ctx> CodeGen__<'ctx> {
    fn jit_compile_sum(&self) -> Option<JitFunction<SumFunc>> {
        let i64_type = self.context.i64_type();
        let fn_type = i64_type.fn_type(
            &[i64_type.into(), i64_type.into(), i64_type.into()],
            false,
        );
        let function = self.module.add_function("sum", fn_type, None);
        let base_block = self.context.append_basic_block(function, "entry");

        self.builder.position_at_end(base_block);

        let x = function.get_nth_param(0).unwrap().into_int_value();
        let y = function.get_nth_param(1).unwrap().into_int_value();
        let z = function.get_nth_param(2).unwrap().into_int_value();

        let sum = self.builder.build_int_add(x, y, "sum");
        let sum = self.builder.build_int_add(sum, z, "sum");

        self.builder.build_return(Some(&sum));

        unsafe { self.execution_engine.get_function("sum").ok() }
    }
}

pub fn test() -> Result<(), Box<dyn Error>> {
    let context = Context::create();
    let module = context.create_module("sum");
    let builder = context.create_builder();
    let execution_engine =
        module.create_jit_execution_engine(OptimizationLevel::None)?;
    let codegen = CodeGen__ {
        context: &context,
        module,
        builder,
        execution_engine,
    };

    let sum = codegen
        .jit_compile_sum()
        .ok_or("Unable to JIT compile `sum`")?;

    let x = 2i64;
    let y = 4i64;
    let z = 6i64;

    unsafe {
        let x = sum.call(x, y, z);
        println!("result: {}", x);
    };

    Ok(())
}
