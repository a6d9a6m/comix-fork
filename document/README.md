# document/ 文档总览

本目录是 Comix 的设计与实现文档，覆盖架构、内核子系统、设备/驱动、用户态接口与工具脚本。本文面向维护者，说明文档范围、组织方式与更新流程。

## 1. 目录结构约定

- `arch/`：架构相关（RISC-V、LoongArch 等）
- `kernel/`：任务、调度、陷阱等内核核心
- `mm/`：内存管理
- `vfs/` 与 `fs/`：虚拟文件系统与具体实现
- `sync/`：同步与并发
- `ipc/`：进程间通信
- `net/`：网络栈与实现指南
- `devices/`：设备与驱动
- `syscall/`：系统调用速查
- `scripts/`：文档维护脚本与工具说明

子目录应尽量提供 `README.md` 作为入口，并在 `SUMMARY.md` 中挂载。

## 2. 写作与更新原则

- 以代码为准：描述应对应当前实现与文件路径，避免泛化成“教程”。
- 先结论后细节：先写接口与行为，再写背景与边界条件。
- 引用源码路径：使用仓库根相对路径，例如 `os/src/arch/riscv/boot/mod.rs`。
- 变更同步：任何关键逻辑改动，应同步更新对应文档与 `SUMMARY.md`。

## 3. 预览与校验

使用 mdBook 预览（可选）：

- 安装：`cargo install mdbook`
- 预览：`mdbook serve document -n 0.0.0.0 -p 4000`
- 构建：`mdbook build document`

脚本参考：
- `document/scripts/style-check.md`
- `document/scripts/rewrite_links.md`

## 4. 文档重写节奏

当目录内容长期未更新时，建议按依赖顺序重写：

1. `README.md` / `SUMMARY.md`
2. `arch/`
3. `kernel/`、`mm/`、`vfs/`、`fs/`
4. `sync/`、`ipc/`、`net/`、`devices/`
5. `syscall/`、`api.md`、`scripts/`

