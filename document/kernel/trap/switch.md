# 特权级切换与返回路径

本页描述用户态与内核态之间的切换路径（以 RISC-V 为主）。

## 1. 用户态 -> 内核态

触发条件:
- 系统调用 (ecall)
- 异常 (页错误/非法指令)
- 中断 (定时器/外部/软件)

硬件行为:
- 进入 S 模式
- 保存 `sepc` 与 `sstatus`
- 跳转到 `stvec` (trap_entry)

## 2. 内核态 -> 用户态

返回路径:
- `trap_entry.S` 的 `__restore` 恢复寄存器
- 执行 `sret` 恢复到 `sepc`

关键点:
- `sstatus.SPP` 决定返回特权级
- `sstatus.SPIE` 恢复中断使能状态

## 3. execve 的切换

`execve` 会更新 TrapFrame，使任务从新入口开始执行:
- 写入入口 `sepc`
- 写入用户栈 `sp`
- 设置 `a0/a1/a2` 传入参数

相关代码:
- `os/src/kernel/task/task_struct.rs`
- `os/src/arch/riscv/trap/trap_frame.rs`

## 4. 关键文件

- `os/src/arch/riscv/trap/trap_entry.S`
- `os/src/arch/riscv/trap/trap_handler.rs`

