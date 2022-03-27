// import initWasm, { format, parse } from 'https://cdn.jsdelivr.net/npm/@x-lang/tools/tools.js';
import initWasm, { format, parse } from '../wasm/tools/pkg/tools.js'; // dev mode
import throttle from 'https://cdn.jsdelivr.net/npm/lodash-es@4.17.21/throttle.js';
import debounce from 'https://cdn.jsdelivr.net/npm/lodash-es@4.17.21/debounce.js';
import { MonokaiTheme } from './theme.js';
import { defaultCode } from './code.js';
import { getErrorStack, clearErrorStack } from './helper.js';

monaco.editor.defineTheme('default-theme', MonokaiTheme);
monaco.editor.setTheme('default-theme');

const jsonViewer = document.getElementById('json-viewer');
const errorOverlay = document.getElementById('error-overlay');

const editor = monaco.editor.create(document.getElementById('editor'), {
    language: 'xlang',
    value: defaultCode,
    minimap: { enabled: false },
    fontSize: 13,
    lineHeight: 22,
    tabSize: 4
});

function clearError() {
    errorOverlay.style.display = "none";
    clearErrorStack();
}

function putError() {
    errorOverlay.style.display = "block";
    errorOverlay.innerHTML = getErrorStack().join('\n');
}

const parseCode = () => {
    clearError();

    const code = editor.getValue();
    try {
        const ast = JSON.parse(parse(code));
        console.log('【PARSE AST】', ast);
        jsonViewer.data = ast;

    } catch (e) {
        putError();
        console.error(e && e.stack);
    }
};

editor.onDidChangeModelContent(debounce(() => {
    parseCode();
}, 200));

window.addEventListener('resize', throttle(() => {
    editor.layout();
}, 50));

document.getElementById('btn-format').addEventListener('click', () => {
    const formattedCode = format(editor.getValue());
    editor.setValue(formattedCode);
});
document.getElementById('btn-collapse').addEventListener('click', () => {
    jsonViewer.collapseAll();
});
document.getElementById('btn-expand').addEventListener('click', () => {
    jsonViewer.expandAll();
});

// 加载 wasm 完成
initWasm().then(() => {
    document.querySelector('.app-loading').style.display = 'none';
    parseCode();
    jsonViewer.expand("*.*.*.*");
})