use std::process::Command;

// TODO: 本地 llvm-config 命令路径
const LLVM_CONFIG_PATH: &str = "/usr/local/llvm/bin/llvm-config";

fn run_command(cmd: &str, args: Vec<&str>) -> Vec<String> {
    let output = Command::new(cmd)
        .args(args)
        .output()
        .expect(&format!("{} not found", cmd))
        .stdout;
    let output_str = std::str::from_utf8(&output).unwrap();
    let list = output_str.split_whitespace();

    let mut result = vec![];
    for i in list {
        result.push(i.to_string());
    }
    result
}

fn main() {
    let mut cfg = cc::Build::new();
    cfg.warnings(false);

    // link LLVM
    let cxxflags = run_command(LLVM_CONFIG_PATH, vec![
        "--cxxflags", "--ldflags", "--system-libs", "--libs", "core",
    ]);
    for i in cxxflags.iter() {
        cfg.flag(i);
    }

    cfg.cpp(true)
        .file("llvm-wrapper/lib.cpp")
        .compile("llvm-wrapper")
}