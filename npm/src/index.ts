import {loadBinding} from '@node-rs/helper';
import path from 'path';
import {Node} from './types';

const bindings = loadBinding(
    path.join(__dirname, '..'),
    'x-lang',
    '@x-lang/core'
);

const xlang = {
    /**
     * 版本
     */
    version: require('../package.json').version as string,

    /**
     * 解析成 AST
     * @param input
     */
    parse: (input: string): Node => {
        return bindings.parse(String(input || ''));
    },

    /**
     * 遍历 AST
     * @param ast
     */
    walk: (ast: Node) => {
    }
}

export = xlang;

