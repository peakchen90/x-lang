use crate::shared::Kind;

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
        argument: Option<Box<Node>>,
    },
    ExpressionStatement {
        expression: Box<Node>,
    },
    IfStatement {
        condition: Box<Node>,
        consequent: Box<Node>,
        alternate: Option<Box<Node>>,
    },
    LoopStatement {
        label: Option<String>,
        body: Box<Node>,
    },
    BreakStatement {
        label: Option<String>,
    },
    ContinueStatement {
        label: Option<String>,
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
    UnaryExpression {
        argument: Box<Node>,
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
    NumberLiteral {
        value: f64,
    },
    BooleanLiteral {
        value: bool,
    },
    // StringLiteral {
    //     value: String
    // },
}

impl Node {
    // 读取一个数字节点的值
    pub fn read_number(&self) -> f64 {
        match self {
            Node::NumberLiteral { value } => *value,
            _ => panic!("Error"),
        }
    }

    // 读取一个布尔节点的值
    pub fn read_bool(&self) -> bool {
        match self {
            Node::BooleanLiteral { value } => *value,
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

    // 读取块语句的 body
    pub fn read_block_body(&self) -> &Vec<Box<Node>> {
        match self {
            Node::BlockStatement { body } => body,
            _ => panic!("Error"),
        }
    }
}
