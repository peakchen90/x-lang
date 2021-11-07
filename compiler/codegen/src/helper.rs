use inkwell::types::{ FloatType, VoidType};
use inkwell::values::FloatValue;

/// LLVM 数据类型
#[derive(Debug)]
pub enum LLVMDataType<'ctx> {
    Number(FloatType<'ctx>),
    Void(VoidType<'ctx>),
}

impl<'ctx> LLVMDataType<'ctx> {
    pub fn get_number_type(&self) -> &FloatType<'ctx> {
        match self {
            LLVMDataType::Number(v) => v,
            _ => panic!("Error"),
        }
    }

    pub fn get_void_type(&self) -> &VoidType<'ctx> {
        match self {
            LLVMDataType::Void(v) => v,
            _ => panic!("Error"),
        }
    }
}

/// LLVM 数据值
#[derive(Debug)]
pub enum LLVMDataValue<'ctx> {
    Number(FloatValue<'ctx>),
    Void,
}

impl<'ctx> LLVMDataValue<'ctx> {
    pub fn read_number(&self) -> &FloatValue<'ctx> {
        match self {
            LLVMDataValue::Number(v) => v,
            _ => panic!("Error"),
        }
    }
}
