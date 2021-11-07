// 关键字
const KEYWORDS: [&str; 3] = ["fn", "var", "return"];

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

#[derive(Debug, PartialEq, Eq, Serialize, Copy, Clone)]
pub enum KindName {
    Number,
    Void,
}

impl KindName {
    // 通过字符串创建 KindName，无效类型将会抛错
    pub fn get(kind: &str, allow_void: bool) -> Self {
        if kind == "num" {
            KindName::Number
        } else if kind == "void" {
            if !allow_void {
                panic!("Unexpected kind: {}", kind);
            }
            KindName::Void
        } else {
            panic!("Invalid kind: {}", kind)
        }
    }

    // 返回类型名称字符串
    pub fn to_string(&self) -> String {
        match self {
            KindName::Number => "num".to_string(),
            KindName::Void => "void".to_string(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Copy, Clone)]
pub enum Kind {
    Some(KindName),
    Infer, // 推断的类型
    None,  // 无类型 或者 void
}

impl Kind {
    // 类型是否是精确的
    pub fn is_exact(&self) -> bool {
        if let Kind::Some(_) = self {
            true
        } else {
            false
        }
    }

    // 读取 KindName
    pub fn read_kind_name(&self) -> Option<&KindName> {
        if let Kind::Some(v) = self {
            Some(v)
        } else {
            None
        }
    }

    // 返回类型字符串，非精确的类型返回 ""
    pub fn to_string(&self) -> String {
        if let Kind::Some(v) = self {
            v.to_string()
        } else {
            String::from("")
        }
    }
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
    // 返回 node 类型名
    pub fn node_type(&self) -> String {
        let str = match self {
            Node::Program { .. } => "Program",
            Node::FunctionDeclaration { .. } => "FunctionDeclaration",
            Node::VariableDeclaration { .. } => "VariableDeclaration",
            Node::BlockStatement { .. } => "BlockStatement",
            Node::ReturnStatement { .. } => "ReturnStatement",
            Node::ExpressionStatement { .. } => "ExpressionStatement",
            Node::CallExpression { .. } => "CallExpression",
            Node::BinaryExpression { .. } => "BinaryExpression",
            Node::AssignmentExpression { .. } => "AssignmentExpression",
            Node::Identifier { .. } => "Identifier",
            Node::NumberLiteral { .. } => "NumberLiteral",
        };
        str.to_string()
    }

    // 读取一个数字节点的值
    pub fn read_number(&self) -> f64 {
        match self {
            Node::NumberLiteral { value } => *value,
            _ => panic!("Error"),
        }
    }

    // 读取一个标识符的名称及类型
    pub fn read_identifier(&self) -> (&str, &Kind) {
        match self {
            Node::Identifier { name, kind } => (name, kind),
            _ => panic!("Error"),
        }
    }
}
