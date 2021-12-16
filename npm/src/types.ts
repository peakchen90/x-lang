export interface Position {
    start: number
    end: number
}

export type Kind = 'number' | 'boolean' | 'string' | 'void' | 'infer' | null;

interface BaseNode {
    type: string
    position: Position
}

interface Program extends BaseNode {
    type: 'Program'
    body: Node[]
}

interface ImportDeclaration extends BaseNode {
    type: 'ImportDeclaration'
    source: string
    isStdSource: boolean
    specifiers: Node[] | null
}

interface FunctionDeclaration extends BaseNode {
    type: 'FunctionDeclaration'
    id: Node
    arguments: Node[]
    body: Node
    returnKind: Kind
    isPub: boolean
}

interface VariableDeclaration extends BaseNode {
    type: 'VariableDeclaration'
    id: Node
    init: Node
}

interface BlockStatement extends BaseNode {
    type: 'BlockStatement'
    body: Node[]
}

interface ReturnStatement extends BaseNode {
    type: 'ReturnStatement'
    argument: Node | null
}

interface ExpressionStatement extends BaseNode {
    type: 'ExpressionStatement'
    expression: Node
}

interface IfStatement extends BaseNode {
    type: 'IfStatement'
    condition: Node
    consequent: Node
    alternate: Node | null
}

interface LoopStatement extends BaseNode {
    type: 'LoopStatement'
    label: string | null
    body: Node
}

interface BreakStatement extends BaseNode {
    type: 'BreakStatement'
    label: string | null
}

interface ContinueStatement extends BaseNode {
    type: 'ContinueStatement'
    label: string | null
}

interface ImportSpecifier extends BaseNode {
    type: 'ImportSpecifier'
    imported: string
    local: string | null
}

interface CallExpression extends BaseNode {
    type: 'CallExpression'
    callee: Node
    arguments: Node[]
}

interface BinaryExpression extends BaseNode {
    type: 'BinaryExpression'
    left: Node
    right: Node
    operator: string
}

interface UnaryExpression extends BaseNode {
    type: 'UnaryExpression'
    argument: Node
    operator: string
}

interface AssignmentExpression extends BaseNode {
    type: 'AssignmentExpression'
    left: Node
    right: Node
    operator: string
}

interface Identifier extends BaseNode {
    type: 'Identifier'
    name: string
    kind: Kind
}

interface NumberLiteral extends BaseNode {
    type: 'NumberLiteral'
    value: number
}

interface BooleanLiteral extends BaseNode {
    type: 'BooleanLiteral'
    value: string
}

interface StringLiteral extends BaseNode {
    type: 'StringLiteral'
    value: number
    isRaw: boolean
}

export type Node =
    Program | ImportDeclaration | FunctionDeclaration | VariableDeclaration |
    BlockStatement | ReturnStatement | ExpressionStatement | IfStatement |
    LoopStatement | BreakStatement | ContinueStatement | ImportSpecifier |
    CallExpression | BinaryExpression | UnaryExpression | AssignmentExpression |
    Identifier | NumberLiteral | BooleanLiteral | StringLiteral
