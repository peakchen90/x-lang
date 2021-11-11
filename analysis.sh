#!/bin/sh

# TODO: 需依赖工具 http://www.graphviz.org/

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
done