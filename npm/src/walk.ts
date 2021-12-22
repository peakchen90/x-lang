import {Node, WalkContext, WalkVisitor, WalkVisitorType} from './types';

/**
 * 遍历 AST
 * @param node
 * @param visitor
 */
export function walk(node: Node, visitor: WalkVisitor): void {
    if (!node || !visitor) {
        return;
    }

    let isStop = false;
    const state = {};
    const stop = () => isStop = true;

    const walkNode = (node: Node, parent: Node | null) => {
        if (isStop) return;
        const context: WalkContext = {state, parent, stop}

        const type = node.type;
        if (visitor[type]) {
            visitor[type].call(null, node, context);
        }

        switch (node.type) {
            case 'Program':
                for (const stat of node.body) {
                    if (isStop) break;
                    walkNode(stat, node);
                }
                break;
            case 'ImportDeclaration':
                if (node.specifiers) {
                    for (const specifier of node.specifiers) {
                        if (isStop) break;
                        walkNode(specifier, node);
                    }
                }
                break;
            case 'FunctionDeclaration':
                walkNode(node.id, node);
                for (const argument of node.arguments) {
                    if (isStop) break;
                    walkNode(argument, node);
                }
                walkNode(node.body, node);
                break;
            case 'VariableDeclaration':
                walkNode(node.id, node);
                walkNode(node.init, node);
                break;
            case 'BlockStatement':
                for (const stat of node.body) {
                    if (isStop) break;
                    walkNode(stat, node);
                }
                break;
            case 'ReturnStatement':
                if (node.argument) {
                    walkNode(node.argument, node);
                }
                break;
            case 'ExpressionStatement':
                walkNode(node.expression, node);
                break;
            case 'IfStatement':
                walkNode(node.condition, node);
                walkNode(node.consequent, node);
                if (node.alternate) {
                    walkNode(node.alternate, node);
                }
                break;
            case 'LoopStatement':
                walkNode(node.body, node);
                break;
            case 'BreakStatement':
                break;
            case 'ContinueStatement':
                break;
            case 'ImportSpecifier':
                break;
            case 'CallExpression':
                walkNode(node.callee, node);
                for (const argument of node.arguments) {
                    if (isStop) break;
                    walkNode(argument, node);
                }
                break;
            case 'BinaryExpression':
                walkNode(node.left, node);
                walkNode(node.right, node);
                break;
            case 'UnaryExpression':
                walkNode(node.argument, node);
                break;
            case 'AssignmentExpression':
                walkNode(node.left, node);
                walkNode(node.right, node);
                break;
            case 'Identifier':
                break;
            case 'NumberLiteral':
                break;
            case 'BooleanLiteral':
                break;
            case 'StringLiteral':
                break;
            default:
                throw new Error('Unexpected type: ' + type);
        }

        const exitType = `${type}:exit` as WalkVisitorType;
        if (visitor[exitType]) {
            visitor[exitType].call(null, node, context);
        }
    }

    walkNode(node, null);
}
