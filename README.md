# CCYOS

CCYOS 是面向全国大学生操作系统大赛内核赛道的教学与竞赛内核项目，主要用于组织与验证内核设计、实现与测试流程，目标平台以 RISC-V 64 位 QEMU virt 为主。

## 继承与致谢

CCYOS 基于 ComixOS 的开源代码派生与演进。我们在其基础上进行功能扩展、架构调整与工程化改进，并保留原有的开源协议与版权信息。感谢 ComixOS 社区的贡献与积累。

## 仓库布局

- document/：设计文档与开发指南（mdBook 结构）
- os/：内核 crate 与构建脚本（Makefile、build.rs、链接脚本等）
- user/：用户态支持库与示例程序（自动被构建并打包）
- data/：根文件系统基础内容（busybox、init、配置等）
- scripts/：工具脚本（镜像打包、链接重写等）
- Makefile：顶层便捷命令（build/run/clean、Docker 构建）

## 环境依赖

- Rust nightly（已在 rust-toolchain.toml 固定）
- RISC-V 目标：`rustup target add riscv64gc-unknown-none-elf`
- QEMU：`qemu-system-riscv64`
- 构建工具：`make`、`python3`、`dd`、`mkfs.ext4`、`rust-objcopy`
- 可选：Docker/DevContainer 直接复用仓库提供的镜像

## 构建与运行

```bash
# 构建内核（自动编译 user 程序并生成 fs.img）
make build

# 在 QEMU 运行（使用 VirtIO-Block 挂载 fs.img）
cd os && make run

# 调试：前台等待 GDB
a) cd os && make debug   # 启动 QEMU 等待 :1234
b) cd os && make gdb     # 另一个终端连接 GDB

# 在 QEMU 中运行内核测试
cd os && make test
```

## 文档与贡献

- 阅读文档：参见 [document/README.md](document/README.md) 和 SUMMARY 导航
- 贡献流程：请先阅读 [CONTRIBUTING.md](CONTRIBUTING.md)

## 许可证

[GPL-3.0](LICENSE)
