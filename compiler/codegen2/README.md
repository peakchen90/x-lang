# x-lang-codegen

## 环境准备

### 安装 LLVM 命令行工具
> 教程仅针对 Unix-like 系统

- 打开 [LLVM 下载页面](https://github.com/llvm/llvm-project/releases/)，下载 **llvm-13.0.0.src.tar.xz**（版本号自己选择）
- 下载完成后通过编译安装 ([参考](https://zhuanlan.zhihu.com/p/102028114)) ：
  1. `tar xvf llvm-project-13.0.0.src.tar.xz`：解压文件
  2. `cd llvm-13.0.0.src`
  3. `sudo mkdir -p /usr/local/llvm`: 创建目录，待会会将 llvm 安装到此目录
  4. `mkdir build && cd build`: 创建编译目录 build 并进入目录
  5. `cmake -G "Unix Makefiles" -DLLVM_ENABLE_PROJECTS="" -DLLVM_TARGETS_TO_BUILD=X86 -DCMAKE_BUILD_TYPE="Release" -DLLDB_INCLUDE_TESTS=OFF -DCMAKE_INSTALL_PREFIX="/usr/local/llvm" ../`: 生成编译配置文件
  6. `make -j 6`: 开始编译，`-j 6` 表示可以 6 个任务同时运行，加快编译速度（这里比较费时间）
  7. `sudo make install`: 安装到目标目录
  8. 安装完成后，将 bin 添加到 PATH 环境变量中：`export PATH=/usr/local/llvm/bin:$PATH`
  9. 测试一下是否安装成功：`llvm-config --version`
