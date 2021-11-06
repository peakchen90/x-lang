use std::ops::Deref;

// 关键字
const KEYWORDS: [&str; 3] = ["fn", "var", "return"];

// 数据类型
const KINDS: [&str; 1] = ["num"];

fn array_index_of_str(arr: &[&str], value: &str) -> isize {
    for (i, v) in arr.iter().enumerate() {
        if *v == value {
            return i as isize;
        }
    }
    -1
}

// 判断是否为关键字
pub fn is_keyword_str(str: &str) -> bool {
    array_index_of_str(&KEYWORDS, str) >= 0
}

// 判断是否为数据类型字符串
pub fn is_kind_str(str: &str) -> bool {
    array_index_of_str(&KINDS, str) >= 0
}

// 校验数据类型标识符是否正确，不正确抛出错误
pub fn validate_kind(kind: &str) {
    if !is_kind_str(kind) {
        panic!("Invalid kind: {}", kind);
    }
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub enum Kind {
    Some(String),
    Unknown, // 暂时未知的类型
    None,    // 无类型（标识符）
}

#[derive(Debug, PartialEq, Eq)]
pub enum TokenType {
    Keyword,
    Identifier,
    Number,
    EOF,
    Assign,    // =
    Plus,      // +
    Sub,       // -
    Mul,       // *
    Div,       // /
    ParenL,    // (
    ParenR,    // )
    BraceL,    // {
    BraceR,    // }
    Comma,     // ,
    Semi,      // ;
    Colon,     // :
    ReturnSym, // ->
}

#[derive(Debug, Serialize)]
pub enum Node {
    Program {
        body: Vec<Box<Node>>,
    },

    //  statements
    FunctionDeclaration {
        id: Box<Node>,
        arguments: Vec<Box<Node>>,
        body: Box<Node>,
        return_kind: Kind,
    },
    VariableDeclaration {
        id: Box<Node>,
        init: Box<Node>,
    },
    BlockStatement {
        body: Vec<Box<Node>>,
    },
    ReturnStatement {
        argument: Box<Node>,
    },
    ExpressionStatement {
        expression: Box<Node>,
    },

    // expressions
    CallExpression {
        callee: Box<Node>,
        arguments: Vec<Box<Node>>,
    },
    BinaryExpression {
        left: Box<Node>,
        right: Box<Node>,
        operator: String,
    },
    AssignmentExpression {
        left: Box<Node>,
        right: Box<Node>,
        operator: String,
    },
    Identifier {
        name: String,
        kind: Kind,
    },
    // StringLiteral {
    //     value: String
    // },
    NumberLiteral {
        value: f64,
    },
}

impl Node {
    // pub fn read_number_value(&self) -> f64 {
    //     match self {
    //         Node::NumberLiteral { value } => *value,
    //         _ => panic!("Error"),
    //     }
    // }
    //
    // pub fn read_identifier_name<T>(&self) -> &str {
    //     match self {
    //         Node::Identifier { name } => name,
    //         _ => panic!("Error"),
    //     }
    // }
}
