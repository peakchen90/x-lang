use crate::helper::to_kind_str;
use napi::{CallContext, JsObject, JsString};
use std::ops::Deref;
use x_lang_ast::node::Node;
use x_lang_ast::shared::Kind;
use x_lang_ast::state::Parser;

// 解析
#[js_function(1)]
pub fn parse(ctx: CallContext) -> napi::Result<(JsObject)> {
    let input = ctx.get::<JsString>(0)?.into_utf8()?;
    let input = input.as_str()?;

    let parser = Parser::new(input);
    let node = parser.node.unwrap();
    let node = transform_js_ast(&ctx, &node)?;
    Ok(node)
}

fn transform_js_ast(ctx: &CallContext, node: &Node) -> napi::Result<(JsObject)> {
    let set_position = |root: &mut JsObject,
                        position: (usize, usize)|
     -> napi::Result<()> {
        let mut ast_position = ctx.env.create_object()?;
        ast_position
            .set_named_property("start", ctx.env.create_uint32(position.0 as u32)?);
        ast_position.set_named_property("end", ctx.env.create_uint32(position.1 as u32)?);
        root.set_named_property("position", ast_position);
        Ok(())
    };

    match node {
        Node::Program { body, position } => {
            let mut ast_root = ctx.env.create_object()?;
            let mut ast_body = ctx.env.create_array_with_length(body.len())?;
            for (index, stat) in body.iter().enumerate() {
                let ast_stat = transform_js_ast(ctx, stat.deref())?;
                ast_body.set_element(index as u32, ast_stat);
            }

            ast_root.set_named_property("type", ctx.env.create_string("Program")?);
            ast_root.set_named_property("body", ast_body);
            set_position(&mut ast_root, *position);
            Ok(ast_root)
        }
        Node::ImportDeclaration {
            source,
            is_std_source,
            specifiers,
            position,
        } => {
            let mut ast_root = ctx.env.create_object()?;

            ast_root
                .set_named_property("type", ctx.env.create_string("ImportDeclaration")?);
            ast_root.set_named_property("source", ctx.env.create_string(source)?);
            ast_root
                .set_named_property("isStdSource", ctx.env.get_boolean(*is_std_source)?);
            if let Some(specifiers) = specifiers {
                let mut ast_specifiers =
                    ctx.env.create_array_with_length(specifiers.len())?;
                for (index, specifier) in specifiers.iter().enumerate() {
                    let ast_specifier = transform_js_ast(ctx, specifier.deref())?;
                    ast_specifiers.set_element(index as u32, ast_specifier);
                }
                ast_root.set_named_property("specifiers", ast_specifiers);
            } else {
                ast_root.set_named_property("specifiers", ctx.env.get_null()?);
            }
            set_position(&mut ast_root, *position);
            Ok(ast_root)
        }
        Node::FunctionDeclaration {
            id,
            arguments,
            body,
            return_kind,
            is_pub,
            position,
        } => {
            let mut ast_root = ctx.env.create_object()?;
            let mut ast_arguments = ctx.env.create_array_with_length(arguments.len())?;
            for (index, arg) in arguments.iter().enumerate() {
                let ast_arg = transform_js_ast(ctx, arg.deref())?;
                ast_arguments.set_element(index as u32, ast_arg);
            }

            ast_root.set_named_property(
                "type",
                ctx.env.create_string("FunctionDeclaration")?,
            );
            ast_root.set_named_property("id", transform_js_ast(ctx, id.deref())?);
            ast_root.set_named_property("arguments", ast_arguments);
            ast_root.set_named_property("body", transform_js_ast(ctx, body.deref())?);
            if let Some(v) = to_kind_str(return_kind) {
                ast_root.set_named_property("returnKind", ctx.env.create_string(v)?);
            } else {
                ast_root.set_named_property("returnKind", ctx.env.get_null()?);
            }
            ast_root.set_named_property("isPub", ctx.env.get_boolean(*is_pub)?);
            set_position(&mut ast_root, *position);
            Ok(ast_root)
        }
        Node::VariableDeclaration { id, init, position } => {
            let mut ast_root = ctx.env.create_object()?;

            ast_root.set_named_property(
                "type",
                ctx.env.create_string("VariableDeclaration")?,
            );
            ast_root.set_named_property("id", transform_js_ast(ctx, id.deref())?);
            ast_root.set_named_property("init", transform_js_ast(ctx, init.deref())?);
            set_position(&mut ast_root, *position);
            Ok(ast_root)
        }
        Node::BlockStatement { body, position } => {
            let mut ast_root = ctx.env.create_object()?;
            let mut ast_body = ctx.env.create_array_with_length(body.len())?;
            for (index, stat) in body.iter().enumerate() {
                let ast_stat = transform_js_ast(ctx, stat.deref())?;
                ast_body.set_element(index as u32, ast_stat);
            }

            ast_root.set_named_property("type", ctx.env.create_string("BlockStatement")?);
            ast_root.set_named_property("body", ast_body);
            set_position(&mut ast_root, *position);
            Ok(ast_root)
        }
        Node::ReturnStatement { argument, position } => {
            let mut ast_root = ctx.env.create_object()?;

            ast_root
                .set_named_property("type", ctx.env.create_string("ReturnStatement")?);
            if let Some(v) = argument {
                ast_root
                    .set_named_property("argument", transform_js_ast(ctx, v.deref())?);
            } else {
                ast_root.set_named_property("argument", ctx.env.get_null()?);
            }
            set_position(&mut ast_root, *position);
            Ok(ast_root)
        }
        Node::ExpressionStatement {
            expression,
            position,
        } => {
            let mut ast_root = ctx.env.create_object()?;

            ast_root.set_named_property(
                "type",
                ctx.env.create_string("ExpressionStatement")?,
            );
            ast_root.set_named_property(
                "expression",
                transform_js_ast(ctx, expression.deref())?,
            );
            set_position(&mut ast_root, *position);
            Ok(ast_root)
        }
        Node::IfStatement {
            condition,
            consequent,
            alternate,
            position,
        } => {
            let mut ast_root = ctx.env.create_object()?;

            ast_root.set_named_property("type", ctx.env.create_string("IfStatement")?);
            ast_root.set_named_property(
                "condition",
                transform_js_ast(ctx, condition.deref())?,
            );
            ast_root.set_named_property(
                "consequent",
                transform_js_ast(ctx, consequent.deref())?,
            );
            if let Some(v) = alternate {
                ast_root
                    .set_named_property("alternate", transform_js_ast(ctx, v.deref())?);
            } else {
                ast_root.set_named_property("alternate", ctx.env.get_null()?);
            }
            set_position(&mut ast_root, *position);
            Ok(ast_root)
        }
        Node::LoopStatement {
            label,
            body,
            position,
        } => {
            let mut ast_root = ctx.env.create_object()?;

            ast_root.set_named_property("type", ctx.env.create_string("LoopStatement")?);
            if let Some(v) = label {
                ast_root.set_named_property("label", ctx.env.create_string(v)?);
            } else {
                ast_root.set_named_property("label", ctx.env.get_null()?);
            }
            ast_root.set_named_property("body", transform_js_ast(ctx, body.deref())?);
            set_position(&mut ast_root, *position);
            Ok(ast_root)
        }
        Node::BreakStatement { label, position } => {
            let mut ast_root = ctx.env.create_object()?;

            ast_root.set_named_property("type", ctx.env.create_string("BreakStatement")?);
            if let Some(v) = label {
                ast_root.set_named_property("label", ctx.env.create_string(v)?);
            } else {
                ast_root.set_named_property("label", ctx.env.get_null()?);
            }
            set_position(&mut ast_root, *position);
            Ok(ast_root)
        }
        Node::ContinueStatement { label, position } => {
            let mut ast_root = ctx.env.create_object()?;

            ast_root
                .set_named_property("type", ctx.env.create_string("ContinueStatement")?);
            if let Some(v) = label {
                ast_root.set_named_property("label", ctx.env.create_string(v)?);
            } else {
                ast_root.set_named_property("label", ctx.env.get_null()?);
            }
            set_position(&mut ast_root, *position);
            Ok(ast_root)
        }
        Node::ImportSpecifier {
            imported,
            local,
            position,
        } => {
            let mut ast_root = ctx.env.create_object()?;

            ast_root
                .set_named_property("type", ctx.env.create_string("ImportSpecifier")?);
            ast_root.set_named_property("imported", ctx.env.create_string(imported)?);
            if let Some(v) = local {
                ast_root.set_named_property("local", ctx.env.create_string(v)?);
            } else {
                ast_root.set_named_property("local", ctx.env.get_null()?);
            }
            set_position(&mut ast_root, *position);
            Ok(ast_root)
        }
        Node::CallExpression {
            callee,
            arguments,
            position,
        } => {
            let mut ast_root = ctx.env.create_object()?;
            let mut ast_arguments = ctx.env.create_array_with_length(arguments.len())?;
            for (index, arg) in arguments.iter().enumerate() {
                let ast_arg = transform_js_ast(ctx, arg.deref())?;
                ast_arguments.set_element(index as u32, ast_arg);
            }

            ast_root.set_named_property("type", ctx.env.create_string("CallExpression")?);
            ast_root.set_named_property("callee", transform_js_ast(ctx, callee.deref())?);
            ast_root.set_named_property("arguments", ast_arguments);
            set_position(&mut ast_root, *position);
            Ok(ast_root)
        }
        Node::BinaryExpression {
            left,
            right,
            operator,
            position,
        } => {
            let mut ast_root = ctx.env.create_object()?;

            ast_root
                .set_named_property("type", ctx.env.create_string("BinaryExpression")?);
            ast_root.set_named_property("left", transform_js_ast(ctx, left.deref())?);
            ast_root.set_named_property("right", transform_js_ast(ctx, right.deref())?);
            ast_root.set_named_property("operator", ctx.env.create_string(operator)?);
            set_position(&mut ast_root, *position);
            Ok(ast_root)
        }
        Node::UnaryExpression {
            argument,
            operator,
            position,
        } => {
            let mut ast_root = ctx.env.create_object()?;

            ast_root
                .set_named_property("type", ctx.env.create_string("UnaryExpression")?);
            ast_root
                .set_named_property("argument", transform_js_ast(ctx, argument.deref())?);
            ast_root.set_named_property("operator", ctx.env.create_string(operator)?);
            set_position(&mut ast_root, *position);
            Ok(ast_root)
        }
        Node::AssignmentExpression {
            left,
            right,
            operator,
            position,
        } => {
            let mut ast_root = ctx.env.create_object()?;

            ast_root
                .set_named_property("type", ctx.env.create_string("BinaryExpression")?);
            ast_root.set_named_property("left", transform_js_ast(ctx, left.deref())?);
            ast_root.set_named_property("right", transform_js_ast(ctx, right.deref())?);
            ast_root.set_named_property("operator", ctx.env.create_string(operator)?);
            set_position(&mut ast_root, *position);
            Ok(ast_root)
        }
        Node::Identifier {
            name,
            kind,
            position,
        } => {
            let mut ast_root = ctx.env.create_object()?;

            ast_root.set_named_property("type", ctx.env.create_string("Identifier")?);
            ast_root.set_named_property("name", ctx.env.create_string(name)?);
            if let Some(v) = to_kind_str(kind) {
                ast_root.set_named_property("kind", ctx.env.create_string(v)?);
            } else {
                ast_root.set_named_property("kind", ctx.env.get_null()?);
            }
            set_position(&mut ast_root, *position);
            Ok(ast_root)
        }
        Node::NumberLiteral { value, position } => {
            let mut ast_root = ctx.env.create_object()?;

            ast_root.set_named_property("type", ctx.env.create_string("NumberLiteral")?);
            ast_root.set_named_property("value", ctx.env.create_double(*value)?);
            set_position(&mut ast_root, *position);
            Ok(ast_root)
        }
        Node::BooleanLiteral { value, position } => {
            let mut ast_root = ctx.env.create_object()?;

            ast_root.set_named_property("type", ctx.env.create_string("BooleanLiteral")?);
            ast_root.set_named_property("value", ctx.env.get_boolean(*value)?);
            set_position(&mut ast_root, *position);
            Ok(ast_root)
        }
        Node::StringLiteral {
            value,
            is_raw,
            position,
        } => {
            let mut ast_root = ctx.env.create_object()?;

            ast_root.set_named_property("type", ctx.env.create_string("StringLiteral")?);
            ast_root.set_named_property("isRaw", ctx.env.get_boolean(*is_raw)?);
            ast_root.set_named_property("value", ctx.env.create_string(value)?);
            set_position(&mut ast_root, *position);
            Ok(ast_root)
        }
    }
}
