import {loadBinding} from "@node-rs/helper";

const bindings = loadBinding(__dirname, 'x-lang', '@x-lang/core');

export const version = require('./package.json').version;