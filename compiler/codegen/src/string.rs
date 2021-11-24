use crate::utils::{encode_utf16_str, get_string_utf16_chars};
use crate::Compiler;
use inkwell::types::*;
use inkwell::values::{
    AggregateValue, ArrayValue, BasicValue, FloatValue, InstructionOpcode,
    InstructionValue, IntValue, PointerValue, StructValue,
};
use inkwell::AddressSpace;

impl<'ctx> Compiler<'ctx> {
    // 构建存储 string 到内存中, 返回内存地址的值（i64）
    // 使用 utf-16 编码
    pub fn build_string_value(&self, value: &str) -> IntValue<'ctx> {
        let chars = get_string_utf16_chars(value);
        let size = chars.len() as u32;

        let i16_type = self.context.i16_type();
        let i32_type = self.context.i32_type();
        let array_type = i16_type.array_type(size);

        let elements = chars
            .iter()
            .map(|v| i16_type.const_int(*v as u64, false))
            .collect::<Vec<IntValue>>();
        let array_value = i16_type.const_array(elements.as_slice());

        // string struct { size i32, array i16* }
        let str_struct = self
            .context
            .struct_type(&[i32_type.into(), array_type.into()], false);

        // Note: chars 包含了结尾的结束符 0，真实 size 比 chars 长度少 1
        let str_struct_value = str_struct.const_named_struct(&[
            self.context
                .i32_type()
                .const_int((size - 1) as u64, false)
                .into(),
            array_value.into(),
        ]);

        // 分配内存
        let ptr = self.builder.build_alloca(str_struct, "string");
        self.builder.build_store(ptr, str_struct_value);

        self.builder
            .build_ptr_to_int(ptr, self.build_store_ptr_type(), "")
    }

    // 构建读取字符串 size (i32)
    pub fn build_read_string_size(&self, ptr: PointerValue<'ctx>) -> IntValue<'ctx> {
        let size_ptr = self
            .builder
            .build_struct_gep(ptr, 0, "size")
            .expect("Internal Error: string");
        self.builder.build_load(size_ptr, "").into_int_value()
    }

    // 构建读取字符数组 ptr (i16*)
    pub fn build_read_string_ptr(&self, ptr: PointerValue<'ctx>) -> PointerValue<'ctx> {
        let string_arr_ptr = self
            .builder
            .build_struct_gep(ptr, 1, "string_arr")
            .expect("Internal Error: string");
        self.builder.build_pointer_cast(
            string_arr_ptr,
            self.context.i16_type().ptr_type(AddressSpace::Generic),
            "",
        )
    }

    // 构建内存地址 (i64) 转 ptr 值
    pub fn build_cast_string_address(
        &self,
        address: IntValue<'ctx>,
    ) -> PointerValue<'ctx> {
        self.builder.build_int_to_ptr(
            address,
            self.context
                .struct_type(
                    &[
                        self.context.i32_type().into(),
                        self.context.i16_type().array_type(0).into(),
                    ],
                    false,
                )
                .ptr_type(AddressSpace::Generic),
            "",
        )
    }

    // 拷贝一个字符串，并返回新字符串的 ptr
    pub fn build_copy_string(&self, ptr: PointerValue<'ctx>) {
        // self.builder.build_memcpy()
    }

    // TODO: 临时测试
    pub fn string_test(&self) {
        let fn_type = self.build_void_type().fn_type(&[], false);
        let fn_val = self.module.add_function("main", fn_type, None);
        let block = self.context.append_basic_block(fn_val, "entry");
        self.builder.position_at_end(block);
        let print_fn = self.print_fns.get("str").unwrap();
        let print_eol_fn = self.print_fns.get("newline").unwrap();
        let print_int_fn = self.print_fns.get("i64").unwrap();
        let print_num_fn = self.print_fns.get("num").unwrap();

        //////// begin ///////

        let ty = self.build_store_ptr_type();
        let ptr = self.builder.build_alloca(ty, "ptr_store");

        let ty2 = self.build_number_type();
        let ptr2 = self.builder.build_alloca(ty2, "float");
        self.builder.build_store(ptr2, ty2.const_float(6.8));

        let a = self.builder.build_ptr_to_int(ptr2, ty, "aa");
        self.builder.build_store(ptr, a);

        let address = self.builder.build_load(ptr, "").into_int_value();
        let a_ptr = self.builder.build_int_to_ptr(
            address,
            ty2.ptr_type(AddressSpace::Generic),
            "hhh",
        );
        let vv = self.builder.build_load(a_ptr, "==");

        self.builder
            .build_call(*print_int_fn, &[address.into()], "");
        self.builder.build_call(*print_eol_fn, &[], "");
        self.builder.build_call(*print_num_fn, &[vv.into()], "");

        self.builder.build_return(None);

        if !fn_val.verify(true) {
            println!("\n====================== FAIL ==========================");
            println!("====================== FAIL ==========================");
            println!("====================== FAIL ==========================");
            println!("====================== FAIL ==========================");
        }

        unsafe {
            // 读取 main 函数并调用
            type MainFunction = unsafe extern "C" fn() -> isize;
            let main_fn = self.execution_engine.get_function::<MainFunction>("main");
            if let Ok(main_fn) = main_fn {
                main_fn.call();
            }
        }
    }
}
