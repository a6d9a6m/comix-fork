# Trap 处理概述

本页描述内核 Trap 处理流程。实现以 RISC-V 为主，对应代码位于 `os/src/arch/riscv/trap/`。

## 1. 初始化

- `init_boot_trap()`：早期阶段设置 boot trap 入口
- `init()`：设置正式 trap 入口，并启用软件中断 (IPI)

相关代码: `os/src/arch/riscv/trap/mod.rs`

## 2. 进入 trap

入口汇编: `os/src/arch/riscv/trap/trap_entry.S`

关键行为:
- 使用 `sscratch` 获取 TrapFrame 指针
- 保存通用寄存器、`sepc`、`sstatus`
- 写入 `TrapFrame.kernel_sp` 和 `TrapFrame.cpu_ptr`
- 调用 `trap_handler()`

## 3. trap 分发

`trap_handler()` 位于 `os/src/arch/riscv/trap/trap_handler.rs`，核心分支:

- 用户态 `ecall`: 调用 `dispatch_syscall()`
- 定时器中断: `set_next_trigger()` 并驱动调度
- 软件中断: `handle_ipi()`
- 外部中断: 设备处理

## 4. 返回 trap

`trap_entry.S` 的 `__restore` 路径:
- 恢复 `sstatus`/`sepc`
- 恢复通用寄存器
- 执行 `sret`

## 5. 关键文件

- `os/src/arch/riscv/trap/mod.rs`
- `os/src/arch/riscv/trap/trap_entry.S`
- `os/src/arch/riscv/trap/trap_handler.rs`
- `os/src/arch/riscv/trap/trap_frame.rs`

