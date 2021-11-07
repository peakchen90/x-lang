// pub mod compiler;
// pub mod scope;
pub mod helper;
mod test;

pub mod compiler {
    use super::helper::*;
    use crate::test::run_test;
    use libc::*;
    use llvm_sys::core::{LLVMAddFunction, LLVMAddGlobal, LLVMAppendBasicBlockInContext, LLVMBuildAlloca, LLVMBuildCall, LLVMBuildFAdd, LLVMBuildLoad, LLVMBuildRet, LLVMBuildStore, LLVMBuildVAArg, LLVMConstInt, LLVMConstReal, LLVMContextCreate, LLVMCreateBuilder, LLVMDisposeBuilder, LLVMDumpModule, LLVMFloatType, LLVMFloatTypeInContext, LLVMFunctionType, LLVMGetBasicBlockName, LLVMGetNamedFunction, LLVMGetParam, LLVMIntType, LLVMIntTypeInContext, LLVMModuleCreateWithNameInContext, LLVMPointerType, LLVMPositionBuilderAtEnd, LLVMSetValueName2, LLVMVoidType};
    use llvm_sys::execution_engine::{LLVMCreateExecutionEngineForModule, LLVMCreateGenericValueOfFloat, LLVMCreateJITCompilerForModule, LLVMDisposeExecutionEngine, LLVMGenericValueToFloat, LLVMGetFunctionAddress, LLVMRunFunction};
    use std::mem::MaybeUninit;
    use x_lang_ast::shared::Node;

    pub struct Compiler {}

    type AddFunc = unsafe extern "C" fn(f64) -> f64;

    impl Compiler {
        pub fn compile(node: &Node) {
            // run_test();

            println!("=================================================================\n");

            unsafe {
                let context = LLVMContextCreate();
                let module = LLVMModuleCreateWithNameInContext(
                    to_char_ptr("m1"),
                    context,
                );
                let builder = LLVMCreateBuilder();

                let ty = LLVMFloatTypeInContext(context);
                let val = LLVMConstReal(ty, 2.0 as c_double);
                let x = LLVMFunctionType(
                    LLVMFloatType(),
                    [LLVMFloatType()].as_mut_ptr(),
                    1,
                    0,
                );
                let fnv = LLVMAddFunction(module, to_char_ptr("add"), x);
                let a3 = LLVMAppendBasicBlockInContext(
                    context,
                    fnv,
                    to_char_ptr(""),
                );
                LLVMPositionBuilderAtEnd(builder, a3);
                let arg1 = LLVMGetParam(fnv, 0);
                LLVMSetValueName2(arg1, to_char_ptr("n"), 1);
                let ret = LLVMBuildFAdd(
                    builder,
                    arg1,
                    LLVMConstReal(ty, 100 as c_double),
                    to_char_ptr("temp"),
                );
                LLVMBuildRet(builder, ret);

                // call
                // let result = LLVMBuildCall(
                //     builder,
                //     fnv,
                //     [LLVMBuildFAdd(
                //         builder,
                //         arg1,
                //         LLVMConstReal(ty, 8 as c_double),
                //         to_char_ptr("temp"),
                //     )]
                //     .as_mut_ptr(),
                //     1,
                //     to_char_ptr("ccc"),
                // );

                let mut execution_engine = MaybeUninit::uninit();
                let code = LLVMCreateJITCompilerForModule(
                    execution_engine.as_mut_ptr(),
                    module,
                    0,
                    MaybeUninit::uninit().as_mut_ptr(),
                );
                let execution_engine = execution_engine.assume_init();

                let ii = LLVMCreateGenericValueOfFloat(ty, 8 as c_double);

                let x = LLVMGetNamedFunction(module, to_char_ptr("add"));

                let x2 = LLVMGetFunctionAddress(execution_engine, to_char_ptr("add")) as AddFunc;



                // let abc = LLVMDisposeExecutionEngine(execution_engine);
                // let ret = LLVMRunFunction(
                //     execution_engine.as_ref(),
                //     fnv,
                //     1,
                //     [ii].as_mut_ptr(),
                // );

                println!("###result: {}", x2(0.3));

                // let g1 = LLVMAddGlobal(module, LLVMIntType(8), to_char_ptr("ppp"));

                // let ptr = LLVMBuildAlloca(builder, ty, to_char_ptr("var_aa"));
                // LLVMBuildStore(builder, val, ptr);
                // LLVMBuildStore(builder, g1, ptr);
                //

                // LLVMBuildStore(builder, g1, x);
                // x.set_ptr_value("".as_mut_ptr());
                //

                // done building
                LLVMDisposeBuilder(builder);
                // Dump the module as IR to stdout.
                LLVMDumpModule(module);
            }
        }
    }
}
