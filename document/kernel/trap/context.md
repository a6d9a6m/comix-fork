# TrapFrame 上下文

TrapFrame 保存 trap 发生时的完整寄存器状态，布局必须与汇编保存顺序一致。

## 1. 数据结构

定义位置: `os/src/arch/riscv/trap/trap_frame.rs`

关键字段:
- `sepc`: 异常返回地址
- `sstatus`: 中断/特权级状态
- `x1_ra`..`x31_t6`: 通用寄存器快照
- `kernel_sp`: 内核栈指针
- `cpu_ptr`: 当前 CPU 指针，用于恢复内核态 `tp`

## 2. 与 trap_entry 的关系

`trap_entry.S` 负责:
- 将寄存器写入 TrapFrame
- 从 TrapFrame 恢复寄存器
- 利用 `cpu_ptr` 恢复 `tp`

任何字段顺序变化都必须同步更新汇编保存/恢复逻辑。

## 3. 初始化与更新

- `TrapFrame::zero_init()` 会自动填充 `cpu_ptr`
- `set_exec_trap_frame()` 写入用户态入口、栈与参数
- `set_trap_frame_cpu_ptr()` 在任务迁移时更新 `cpu_ptr`

相关代码:
- `os/src/arch/riscv/trap/trap_frame.rs`
- `os/src/arch/riscv/kernel/cpu.rs`

