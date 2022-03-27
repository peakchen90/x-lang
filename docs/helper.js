let errStack = [];

window.__throw__ = (msg) => {
    errStack.push(`Error: ${msg}`);
    throw new Error(msg);
}

window.__logError__ = (msg) => {
    errStack.push(msg);
    console.error(msg);
}

export function getErrorStack() {
    return errStack.slice();
}

export function clearErrorStack() {
    errStack = []
}