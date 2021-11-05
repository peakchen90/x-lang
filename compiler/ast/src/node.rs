#[derive(Debug, Serialize)]
pub enum Node {
    Program {
        body: Vec<Box<Node>>
    },

    //  statements
    FunctionDeclaration {
        id: Box<Node>,
        arguments: Vec<Box<Node>>,
        body: Box<Node>,
    },
    VariableDeclaration {
        id: Box<Node>,
        init: Box<Node>,
    },
    BlockStatement {
        body: Vec<Box<Node>>
    },
    ReturnStatement {
        argument: Box<Node>
    },
    ExpressionStatement {
        expression: Box<Node>
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
        name: String
    },
    // StringLiteral {
    //     value: String
    // },
    NumberLiteral {
        value: f64
    },
}