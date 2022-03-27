export const defaultCode = `
fn sum2(b :num, c :bool) -> num {
    if (c) {
        return b; // comment
    }
    return b + 1;
}

fn main() {
    sum2(0.5, false);
}
`.trim()