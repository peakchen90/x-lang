export interface Position {
    start: number
    end: number
}

export type Kind = 'number' | 'boolean' | 'string' | 'void' | 'infer' | null;

export type NodeType =
    'Program' | 'ImportDeclaration' | 'FunctionDeclaration' | 'VariableDeclaration' |
    'BlockStatement' | 'ReturnStatement' | 'ExpressionStatement' | 'IfStatement' |
    'LoopStatement' | 'BreakStatement' | 'ContinueStatement' | 'ImportSpecifier' |
    'CallExpression' | 'BinaryExpression' | 'UnaryExpression' | 'AssignmentExpression' |
    'Identifier' | 'NumberLiteral' | 'BooleanLiteral' | 'StringLiteral'

export interface BaseNode {
    type: NodeType
    position: Position
}

export interface Program extends BaseNode {
    type: 'Program'
    body: Node[]
}

export interface ImportDeclaration extends BaseNode {
    type: 'ImportDeclaration'
    source: string
    isStdSource: boolean
    specifiers: Node[] | null
}

export interface FunctionDeclaration extends BaseNode {
    type: 'FunctionDeclaration'
    id: Node
    arguments: Node[]
    body: Node
    returnKind: Kind
    isPub: boolean
}

export interface VariableDeclaration extends BaseNode {
    type: 'VariableDeclaration'
    id: Node
    init: Node
}

export interface BlockStatement extends BaseNode {
    type: 'BlockStatement'
    body: Node[]
}

export interface ReturnStatement extends BaseNode {
    type: 'ReturnStatement'
    argument: Node | null
}

export interface ExpressionStatement extends BaseNode {
    type: 'ExpressionStatement'
    expression: Node
}

export interface IfStatement extends BaseNode {
    type: 'IfStatement'
    condition: Node
    consequent: Node
    alternate: Node | null
}

export interface LoopStatement extends BaseNode {
    type: 'LoopStatement'
    label: string | null
    body: Node
}

export interface BreakStatement extends BaseNode {
    type: 'BreakStatement'
    label: string | null
}

export interface ContinueStatement extends BaseNode {
    type: 'ContinueStatement'
    label: string | null
}

export interface ImportSpecifier extends BaseNode {
    type: 'ImportSpecifier'
    imported: string
    local: string | null
}

export interface CallExpression extends BaseNode {
    type: 'CallExpression'
    callee: Node
    arguments: Node[]
}

export interface BinaryExpression extends BaseNode {
    type: 'BinaryExpression'
    left: Node
    right: Node
    operator: string
}

export interface UnaryExpression extends BaseNode {
    type: 'UnaryExpression'
    argument: Node
    operator: string
}

export interface AssignmentExpression extends BaseNode {
    type: 'AssignmentExpression'
    left: Node
    right: Node
    operator: string
}

export interface Identifier extends BaseNode {
    type: 'Identifier'
    name: string
    kind: Kind
}

export interface NumberLiteral extends BaseNode {
    type: 'NumberLiteral'
    value: number
}

export interface BooleanLiteral extends BaseNode {
    type: 'BooleanLiteral'
    value: string
}

export interface StringLiteral extends BaseNode {
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

export interface WalkContext {
    /**
     * 使用方共享的状态
     */
    state: Record<string, any>;

    /**
     * 父节点
     */
    parent: Node | null;

    /**
     * 终止遍历
     */
    stop: () => void
}

export type WalkVisitorType = NodeType |
    'Program:exit' | 'ImportDeclaration:exit' | 'FunctionDeclaration:exit' | 'VariableDeclaration:exit' |
    'BlockStatement:exit' | 'ReturnStatement:exit' | 'ExpressionStatement:exit' | 'IfStatement:exit' |
    'LoopStatement:exit' | 'BreakStatement:exit' | 'ContinueStatement:exit' | 'ImportSpecifier:exit' |
    'CallExpression:exit' | 'BinaryExpression:exit' | 'UnaryExpression:exit' | 'AssignmentExpression:exit' |
    'Identifier:exit' | 'NumberLiteral:exit' | 'BooleanLiteral:exit' | 'StringLiteral:exit'

export type WalkVisitor = Record<WalkVisitorType, (node: Node, context: WalkContext) => void>
