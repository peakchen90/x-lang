#!/bin/bash

# TODO: 需依赖工具 http://www.graphviz.org/
# 首先需要执行 `cargo run` 命令生成 .debug.ll 文件后才能分析
# 可以通过追加 `-o` 参数自动打开 svg 文件, Usage: `./analysis.sh -o`

rm -rf .temp/.*.dot
rm -rf .temp/.*.dot.svg

# 在 .temp 目录生成 .dot 文件，而不影响父进程
mkdir -p ".temp"
(
  cd .temp;
  opt -dot-cfg ../.debug.ll
)

for file in `find .temp -name "*.dot"`
do
  dot $file -Tsvg -o $file.svg
  echo "Generated svg file: $file.svg"
  if [[ $1 == "-o" ]];  then
     open $file.svg
  fi
done