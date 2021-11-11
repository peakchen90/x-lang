use crate::compiler::Compiler;
use crate::helper::never;
use inkwell::values::*;
use inkwell::{AtomicRMWBinOp, FloatPredicate, IntPredicate};
use std::ops::Deref;
use x_lang_ast::node::Node;
use x_lang_ast::shared::{Kind, KindName};

impl<'ctx> Compiler<'ctx> {
    pub fn compile_expression(&self, node: &Node) -> BasicValueEnum<'ctx> {
        match node {
            Node::CallExpression { callee, arguments } => {
                self.compile_call_expression(callee.deref(), arguments)
            }
            Node::BinaryExpression {
                left,
                right,
                operator,
            } => {
                let left_kind = self.infer_expression_kind(left.deref());
                let right_kind = self.infer_expression_kind(right.deref());
                if left_kind != right_kind {
                    panic!("Types of binary expressions are inconsistent");
                }

                let kind_name = *left_kind.read_kind_name().unwrap();
                let left = self.compile_expression(left.deref());
                let right = self.compile_expression(right.deref());
                if kind_name == KindName::Number {
                    self.compile_num_binary_expression(&left, &right, operator)
                } else if kind_name == KindName::Boolean {
                    self.compile_bool_binary_expression(&left, &right, operator)
                } else {
                    panic!("Invalid binary expression")
                }
            }
            Node::AssignmentExpression {
                left,
                right,
                operator,
            } => {
                let (left_var, ..) = left.deref().read_identifier();
                let ptr = self.get_declare_var_ptr(left_var);
                let right = self.compile_expression(right.deref());
                match ptr {
                    Some(ptr) => self.builder.build_store(*ptr, right),
                    None => panic!("Can not assign a value on void type"),
                };
                right
            }
            Node::Identifier { name, .. } => {
                let ptr = self.get_declare_var_ptr(name);
                match ptr {
                    Some(ptr) => self.builder.build_load(*ptr, name),
                    None => panic!("Can not get value on void type"),
                }
            }
            Node::NumberLiteral { value } => {
                self.build_number_value(*value).as_basic_value_enum()
            }
            Node::BooleanLiteral { value } => {
                self.build_bool_value(*value).as_basic_value_enum()
            }
            _ => never(),
        }
    }

    pub fn compile_call_expression(
        &self,
        callee: &Node,
        arguments: &Vec<Box<Node>>,
    ) -> BasicValueEnum<'ctx> {
        let (name, ..) = callee.read_identifier();

        // print 方法调用特殊处理
        if name == "print" && self.scope.search_by_name(name, true).is_none() {
            return self.build_call_system_print(arguments);
        }

        let (fn_value, arg_kind_names, _) = self.get_declare_fn(name);

        // 校验参数
        if arg_kind_names.len() != arguments.len() {
            panic!("Call function `{}` can not match arguments", name);
        }

        let args = arguments.iter();
        let mut i = 0;
        let args = args.map(|arg| {
            let arg = arg.deref();
            let infer_arg_kind = self.infer_expression_kind(arg);
            let value = self.compile_expression(arg);

            // 校验参数
            if !match infer_arg_kind {
                Kind::Some(v) => v == arg_kind_names[i],
                _ => false,
            } {
                panic!("Call function `{}` can not match arguments type", name);
            }
            i += 1;
            BasicMetadataValueEnum::from(value)
        });
        let args = args.collect::<Vec<BasicMetadataValueEnum>>();
        let args = args.as_slice();

        self.build_call_fn(fn_value, args, name)
    }

    // 编译两端为数字类型的二元运算符
    pub fn compile_num_binary_expression(
        &self,
        left: &BasicValueEnum<'ctx>,
        right: &BasicValueEnum<'ctx>,
        operator: &str,
    ) -> BasicValueEnum<'ctx> {
        match operator.as_bytes() {
            b"+" => self
                .builder
                .build_float_add(left.into_float_value(), right.into_float_value(), "ADD")
                .as_basic_value_enum(),
            b"-" => self
                .builder
                .build_float_sub(left.into_float_value(), right.into_float_value(), "SUB")
                .as_basic_value_enum(),
            b"*" => self
                .builder
                .build_float_mul(left.into_float_value(), right.into_float_value(), "MUL")
                .as_basic_value_enum(),
            b"/" => self
                .builder
                .build_float_div(left.into_float_value(), right.into_float_value(), "DIV")
                .as_basic_value_enum(),
            b"%" => self
                .builder
                .build_float_rem(left.into_float_value(), right.into_float_value(), "REM")
                .as_basic_value_enum(),
            /*b"&" => self
                .builder
                .build_atomicrmw(
                    AtomicRMWBinOp::And,
                    left.into_int_value(),
                    right.into_int_value(),
                    "BIT_AND",
                )
                .as_basic_value_enum(),
            b"|" => self
                .builder
                .build_or(left.into_int_value(), right.into_int_value(), "BIT_OR")
                .as_basic_value_enum(),
            b"^" => self
                .builder
                .build_xor(left.into_int_value(), right.into_int_value(), "BIT_XOR")
                .as_basic_value_enum(),*/
            b"<" => self
                .builder
                .build_float_compare(
                    FloatPredicate::OLT,
                    left.into_float_value(),
                    right.into_float_value(),
                    "LT",
                )
                .as_basic_value_enum(),
            b"<=" => self
                .builder
                .build_float_compare(
                    FloatPredicate::OLE,
                    left.into_float_value(),
                    right.into_float_value(),
                    "LE",
                )
                .as_basic_value_enum(),
            b">" => self
                .builder
                .build_float_compare(
                    FloatPredicate::OGE,
                    left.into_float_value(),
                    right.into_float_value(),
                    "GT",
                )
                .as_basic_value_enum(),
            b">=" => self
                .builder
                .build_float_compare(
                    FloatPredicate::OGE,
                    left.into_float_value(),
                    right.into_float_value(),
                    "GE",
                )
                .as_basic_value_enum(),
            b"==" => self
                .builder
                .build_float_compare(
                    FloatPredicate::OEQ,
                    left.into_float_value(),
                    right.into_float_value(),
                    "EQ",
                )
                .as_basic_value_enum(),
            b"!=" => self
                .builder
                .build_float_compare(
                    FloatPredicate::ONE,
                    left.into_float_value(),
                    right.into_float_value(),
                    "NE",
                )
                .as_basic_value_enum(),
            _ => panic!("Invalid binary operator between numbers: `{}`", operator),
        }
    }

    // 编译两端为布尔类型的二元运算符
    pub fn compile_bool_binary_expression(
        &self,
        left: &BasicValueEnum<'ctx>,
        right: &BasicValueEnum<'ctx>,
        operator: &str,
    ) -> BasicValueEnum<'ctx> {
        match operator.as_bytes() {
            b"==" => self
                .builder
                .build_int_compare(
                    IntPredicate::EQ,
                    left.into_int_value(),
                    right.into_int_value(),
                    "EQ",
                )
                .as_basic_value_enum(),
            b"!=" => self
                .builder
                .build_int_compare(
                    IntPredicate::NE,
                    left.into_int_value(),
                    right.into_int_value(),
                    "NE",
                )
                .as_basic_value_enum(),
            b"&&" => self
                .builder
                .build_and(left.into_int_value(), right.into_int_value(), "LOGIC_AND")
                .as_basic_value_enum(),
            b"||" => self
                .builder
                .build_or(left.into_int_value(), right.into_int_value(), "LOGIC_OR")
                .as_basic_value_enum(),
            _ => panic!("Invalid binary operator between numbers: `{}`", operator),
        }
    }
}
