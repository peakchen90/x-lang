use crate::shared::Kind;

#[derive(Debug, Serialize)]
pub enum Node {
    Program {
        body: Vec<Box<Node>>,
        position: (usize, usize),
    },

    //  statements
    ImportDeclaration {
        source: String,
        is_std_source: bool,
        specifiers: Option<Vec<Box<Node>>>,
        position: (usize, usize),
    },
    FunctionDeclaration {
        id: Box<Node>,
        arguments: Vec<Box<Node>>,
        body: Box<Node>,
        return_kind: Kind,
        is_pub: bool,
        position: (usize, usize),
    },
    VariableDeclaration {
        id: Box<Node>,
        init: Box<Node>,
        position: (usize, usize),
    },
    BlockStatement {
        body: Vec<Box<Node>>,
        position: (usize, usize),
    },
    ReturnStatement {
        argument: Option<Box<Node>>,
        position: (usize, usize),
    },
    ExpressionStatement {
        expression: Box<Node>,
        position: (usize, usize),
    },
    IfStatement {
        condition: Box<Node>,
        consequent: Box<Node>,
        alternate: Option<Box<Node>>,
        position: (usize, usize),
    },
    LoopStatement {
        label: Option<String>,
        body: Box<Node>,
        position: (usize, usize),
    },
    BreakStatement {
        label: Option<String>,
        position: (usize, usize),
    },
    ContinueStatement {
        label: Option<String>,
        position: (usize, usize),
    },

    // expressions
    ImportSpecifier {
        imported: String,
        local: Option<String>,
        position: (usize, usize),
    },
    CallExpression {
        callee: Box<Node>,
        arguments: Vec<Box<Node>>,
        position: (usize, usize),
    },
    BinaryExpression {
        left: Box<Node>,
        right: Box<Node>,
        operator: String,
        position: (usize, usize),
    },
    UnaryExpression {
        argument: Box<Node>,
        operator: String,
        position: (usize, usize),
    },
    AssignmentExpression {
        left: Box<Node>,
        right: Box<Node>,
        operator: String,
        position: (usize, usize),
    },
    Identifier {
        name: String,
        kind: Kind,
        position: (usize, usize),
    },
    NumberLiteral {
        value: f64,
        position: (usize, usize),
    },
    BooleanLiteral {
        value: bool,
        position: (usize, usize),
    },
    // StringLiteral {
    //     value: String,
    //     position: (usize, usize)
    // },
}

impl Node {
    // 读取一个数字节点的值
    pub fn read_number(&self) -> f64 {
        match self {
            Node::NumberLiteral { value, .. } => *value,
            _ => panic!("Error"),
        }
    }

    // 读取一个布尔节点的值
    pub fn read_bool(&self) -> bool {
        match self {
            Node::BooleanLiteral { value, .. } => *value,
            _ => panic!("Error"),
        }
    }

    // 读取一个标识符的名称及类型
    pub fn read_identifier(&self) -> (&str, &Kind, usize) {
        match self {
            Node::Identifier {
                name,
                kind,
                position,
            } => (name, kind, position.0),
            _ => panic!("Error"),
        }
    }

    // 读取块语句的 body
    pub fn read_block_body(&self) -> &Vec<Box<Node>> {
        match self {
            Node::BlockStatement { body, .. } => body,
            _ => panic!("Error"),
        }
    }

    // 读取 position
    pub fn read_position(&self) -> (usize, usize) {
        match self {
            Node::Program { position, .. } => *position,
            Node::ImportDeclaration { position, .. } => *position,
            Node::FunctionDeclaration { position, .. } => *position,
            Node::VariableDeclaration { position, .. } => *position,
            Node::BlockStatement { position, .. } => *position,
            Node::ReturnStatement { position, .. } => *position,
            Node::ExpressionStatement { position, .. } => *position,
            Node::IfStatement { position, .. } => *position,
            Node::LoopStatement { position, .. } => *position,
            Node::BreakStatement { position, .. } => *position,
            Node::ContinueStatement { position, .. } => *position,
            Node::ImportSpecifier { position, .. } => *position,
            Node::CallExpression { position, .. } => *position,
            Node::BinaryExpression { position, .. } => *position,
            Node::UnaryExpression { position, .. } => *position,
            Node::AssignmentExpression { position, .. } => *position,
            Node::Identifier { position, .. } => *position,
            Node::NumberLiteral { position, .. } => *position,
            Node::BooleanLiteral { position, .. } => *position,
        }
    }
}
