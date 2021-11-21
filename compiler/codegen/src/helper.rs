use crate::build_in::*;
use crate::compiler::Compiler;
use crate::scope::{FunctionScope, ScopeType};
use inkwell::basic_block::BasicBlock;
use inkwell::comdat::ComdatSelectionKind;
use inkwell::context::Context;
use inkwell::types::*;
use inkwell::values::*;
use std::ops::Deref;
use x_lang_ast::code_frame::print_error_frame;
use x_lang_ast::node::Node;
use x_lang_ast::shared::{Kind, KindName};
use x_lang_ast::visitor::Visitor;

// 永从不会发生，用于避免编译器报错
pub fn never() -> ! {
    panic!("NEVER")
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Terminator {
    None,
    Return,
    Break,
}

impl Terminator {
    pub fn is_terminated(&self) -> bool {
        match self {
            Terminator::None => false,
            Terminator::Return => true,
            Terminator::Break => true,
        }
    }

    pub fn is_return(&self) -> bool {
        match self {
            Terminator::Return => true,
            _ => false,
        }
    }

    pub fn is_break(&self) -> bool {
        match self {
            Terminator::Break => true,
            _ => false,
        }
    }

    pub fn merge(&self, other: Terminator) -> Terminator {
        if self.is_terminated() {
            if self.is_return() {
                self.clone()
            } else {
                other
            }
        } else {
            other
        }
    }
}

impl<'ctx> Compiler<'ctx> {
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
        match return_kind.read_return_kind_name() {
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
        pos: usize,
    ) -> (&Kind, &Option<PointerValue<'ctx>>) {
        let target = self.scope.search_by_name(name, false);
        if target.is_none() {
            self.unexpected_err(pos, &format!("Scope `{}` is not found", name))
        }
        target.unwrap().get_var()
    }

    pub fn get_declare_var_ptr(
        &self,
        name: &str,
        pos: usize,
    ) -> &Option<PointerValue<'ctx>> {
        let (_, ptr) = self.get_declare_var(name, pos);
        ptr
    }

    pub fn get_declare_fn(&self, name: &str, pos: usize) -> &FunctionScope<'ctx> {
        let target = self.scope.search_by_name(name, false);
        if target.is_none() {
            self.unexpected_err(pos, &format!("Function `{}` is not declare", name))
        }
        target.unwrap().get_fn()
    }

    pub fn put_variable(
        &mut self,
        name: &str,
        kind: Kind,
        value: Option<BasicValueEnum<'ctx>>,
        is_arg: bool, // 是否是函数参数
        pos: usize,
    ) {
        let current = self.scope.current().unwrap();
        if current.has(name) {
            self.unexpected_err(pos, &format!("Scope name `{}` is exist", name));
        }

        // 内存中的描述名称
        let mem_name = match is_arg {
            true => format!("ARGUMENT.{}", name),
            false => name.to_string(),
        };
        let ptr = match kind
            .read_kind_name()
            .expect("Can not declare void type variable")
        {
            KindName::Number => {
                let ty = self.build_number_type();
                let ptr = self.builder.build_alloca(ty, &mem_name);
                if let Some(v) = value {
                    self.builder.build_store(ptr, v.into_float_value());
                }
                Some(ptr)
            }
            KindName::Boolean => {
                let ty = self.build_bool_type();
                let ptr = self.builder.build_alloca(ty, &mem_name);
                if let Some(v) = value {
                    self.builder.build_store(ptr, v.into_int_value());
                }
                Some(ptr)
            }
            KindName::Void => None,
        };

        let scope_type = ScopeType::Variable { kind, ptr };
        self.scope.put_variable(name, scope_type);
    }

    pub fn push_block_scope(&mut self, basic_block: BasicBlock<'ctx>) {
        self.scope.push(basic_block);
        self.builder.position_at_end(basic_block);
    }

    pub fn pop_block_scope(&mut self) {
        self.scope.pop();
        if let Some(v) = self.scope.current() {
            if let Some(b) = v.basic_block {
                self.builder.position_at_end(b);
            }
        }
    }

    // 推断表达式的返回类型
    pub fn infer_expression_kind(&self, expr: &Node) -> Kind {
        let mut ret_kind = Kind::None;

        Visitor::walk(expr, &mut |node, mut visitor| match node {
            Node::CallExpression {
                callee, position, ..
            } => {
                let (name, ..) = callee.deref().read_identifier();
                match self.scope.search_by_name(name, false) {
                    Some(v) => {
                        if v.is_fn() {
                            let FunctionScope { return_kind, .. } =
                                self.get_declare_fn(name, position.0);
                            if !return_kind.is_exact()
                                || *return_kind.read_return_kind_name() == KindName::Void
                            {
                                self.unexpected_err(
                                    position.0,
                                    &format!("Unexpected call expression: cannot return void type"),
                                )
                            }
                            ret_kind = *return_kind;
                            visitor.stop();
                        }
                    }
                    _ => self.unexpected_err(
                        position.0,
                        &format!("Function `{}` is not found", name),
                    ),
                }
            }
            Node::BinaryExpression {
                left,
                right,
                operator,
                position,
            } => {
                let left_kind = self.infer_expression_kind(left.deref());
                let right_kind = self.infer_expression_kind(right.deref());
                if left_kind != right_kind {
                    self.unexpected_err(
                        position.0,
                        "Types of binary expressions are inconsistent",
                    );
                }

                let kind_name = *left_kind.read_kind_name().unwrap();
                if kind_name == KindName::Number {
                    match operator.as_bytes() {
                        b"<" | b"<=" | b">" | b">=" | b"==" | b"!=" => {
                            ret_kind = Kind::create("bool")
                        }
                        b"+" | b"-" | b"*" | b"/" | b"%" | b"&" | b"|" | b"^" => {
                            ret_kind = Kind::create("num")
                        }
                        _ => self.unexpected_err(position.0, "Invalid binary expression"),
                    }
                } else if kind_name == KindName::Boolean {
                    match operator.as_bytes() {
                        b"==" | b"!=" | b"&&" | b"||" => ret_kind = Kind::create("bool"),
                        _ => self.unexpected_err(position.0, "Invalid binary expression"),
                    }
                } else {
                    self.unexpected_err(position.0, "Invalid binary expression")
                }

                visitor.stop();
            }
            Node::UnaryExpression {
                argument,
                operator,
                position,
            } => {
                let kind = self.infer_expression_kind(argument.deref());
                let kind_name = *kind.read_kind_name().unwrap();
                if kind_name == KindName::Number {
                    match operator.as_bytes() {
                        b"~" => ret_kind = Kind::create("num"),
                        _ => self.unexpected_err(position.0, "Invalid unary expression"),
                    }
                } else if kind_name == KindName::Boolean {
                    match operator.as_bytes() {
                        b"!" => ret_kind = Kind::create("bool"),
                        _ => self.unexpected_err(position.0, "Invalid unary expression"),
                    }
                } else {
                    self.unexpected_err(position.0, "Invalid unary expression")
                }

                visitor.stop();
            }
            Node::AssignmentExpression { left, .. } => {
                ret_kind = self.infer_expression_kind(left.deref());
                visitor.stop();
            }
            Node::Identifier { name, kind, .. } => match kind {
                Kind::Some(_) => {
                    ret_kind = *kind;
                    visitor.stop();
                }
                Kind::Infer => match self.scope.search_by_name(name, false) {
                    Some(v) => {
                        if v.is_var() {
                            let (kind, ..) = v.get_var();
                            if kind.is_exact() {
                                ret_kind = *kind;
                                visitor.stop();
                            }
                        }
                    }
                    _ => {}
                },
                _ => {}
            },
            Node::NumberLiteral { .. } => {
                ret_kind = Kind::create("num");
                visitor.stop();
            }
            Node::BooleanLiteral { .. } => {
                ret_kind = Kind::create("bool");
                visitor.stop();
            }
            _ => never(),
        });
        ret_kind
    }

    // 打印错误帧信息并抛出异常
    pub fn unexpected_err(&self, pos: usize, msg: &str) -> ! {
        let mut message = msg.to_string();
        let position = print_error_frame(self.source, pos, &message);

        if let Some((line, column)) = position {
            message.push_str(&format!(" ({}:{})", line, column))
        }
        panic!("{}", message)
    }

    // 构建一个数字转整数的转换，并返回转换后值的指针
    pub fn build_cost_int_ptr(&self, value: &BasicValueEnum<'ctx>) -> PointerValue<'ctx> {
        let ptr = self.builder.build_alloca(self.context.i64_type(), "");
        let value = value
            .into_float_value()
            .const_to_signed_int(self.context.i64_type());
        self.builder.build_store(ptr, value);
        ptr
    }

    // 构建一个数字转整数的转换，并返回转换后的值
    pub fn build_cost_int_value(
        &self,
        value: &BasicValueEnum<'ctx>,
    ) -> BasicValueEnum<'ctx> {
        let ptr = self.build_cost_int_ptr(value);
        self.builder.build_load(ptr, "CAST_TEMP")
    }

    // 构建一个整数转浮点数字的转换，并返回转换后的值
    pub fn build_cost_float_value(
        &self,
        value: &BasicValueEnum<'ctx>,
    ) -> BasicValueEnum<'ctx> {
        let ptr = self.builder.build_alloca(self.context.f64_type(), "");
        let value = value
            .into_int_value()
            .const_signed_to_float(self.context.f64_type());
        self.builder.build_store(ptr, value);
        self.builder.build_load(ptr, "CAST_TEMP")
    }

    // built-in
    pub fn inject_build_in(&mut self) {
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
            self.bind_system_print_fn("newline", None, system_print_newline as usize);
        }
    }

    fn bind_system_print_fn(
        &mut self,
        type_name: &'static str,
        arg_type: Option<BasicMetadataTypeEnum<'ctx>>,
        address: usize,
    ) {
        let arg_types = match arg_type {
            Some(v) => vec![v],
            None => vec![],
        };
        let print_fn_value = self.build_fn_value(
            &format!("print_{}", type_name),
            &Kind::create("void"),
            arg_types.as_slice(),
        );
        self.execution_engine
            .add_global_mapping(&print_fn_value, address);

        // 保存信息
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
        let len = arguments.len();
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
                    self.build_call_fn(fn_value, &[arg_value.into()], "CALL.sys_print");
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
