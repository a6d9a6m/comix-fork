# RISC-V 寄存器与 CSR 速览

本页聚焦内核实现实际用到的寄存器和 CSR，避免泛化为全部架构手册内容。

## 1. 通用寄存器 (GPR)

RISC-V 有 32 个通用寄存器 x0-x31，ABI 约定如下。内核中调用约定与用户态一致。

| 编号 | ABI | 作用 | 保存约定 |
| --- | --- | --- | --- |
| x0 | zero | 常量 0，写无效 | - |
| x1 | ra | 返回地址 | 调用者保存 |
| x2 | sp | 栈指针 | 被调用者保存 |
| x3 | gp | 全局指针 | - |
| x4 | tp | 线程指针/CPU 私有数据 | - |
| x5-x7 | t0-t2 | 临时寄存器 | 调用者保存 |
| x8 | s0/fp | 保存寄存器/帧指针 | 被调用者保存 |
| x9 | s1 | 保存寄存器 | 被调用者保存 |
| x10-x11 | a0-a1 | 参数/返回值 | 调用者保存 |
| x12-x17 | a2-a7 | 参数 2-7 | 调用者保存 |
| x18-x27 | s2-s11 | 保存寄存器 | 被调用者保存 |
| x28-x31 | t3-t6 | 临时寄存器 | 调用者保存 |

关键约束
- s0-s11 为被调用者保存，函数入口必须按需保存并在返回前恢复。
- a0 既是返回值寄存器，也常用于系统调用返回。

## 2. PC (程序计数器)

PC 不是通用寄存器，无法直接读写。它通过异常入口或跳转指令隐式更新。内核在陷阱保存中使用 sepc 保存 PC。

## 3. S 模式常用 CSR

下表为内核实际依赖的 CSR。更多细节以 RISC-V Privileged Spec 为准。

| CSR | 作用 | 内核用法 |
| --- | --- | --- |
| sstatus | S 模式状态寄存器 | 管理中断开关、特权级与 SUM |
| sepc | 异常返回地址 | 保存 trap 前 PC |
| scause | trap 原因 | 分类中断/异常 |
| stval | 异常附加值 | 记录故障地址等 |
| stvec | trap 入口 | 指向 trap_entry | 
| sscratch | trap 临时指针 | 指向 TrapFrame |
| satp | 页表控制 | 切换地址空间 |
| sie | S 模式中断使能 | 开关 SSIP/STIP/SEIP |
| sip | S 模式中断挂起 | 查询/清除 SSIP |

### 3.1 sstatus 关键位

- SIE: 全局中断使能位 (bit 1)
- SPIE: 上一次中断使能 (bit 5)
- SPP: trap 前特权级 (bit 8)
- SUM: S 模式访问用户内存许可 (内核访问用户内存时打开)

代码位置参考
- `os/src/arch/riscv/constant.rs` 定义 SIE/SPIE/SPP 位掩码
- `os/src/arch/riscv/trap/sum_guard.rs` 管理 SUM 位

### 3.2 sie/sip 与软件中断

本项目在 IPI 中使用 SSIP 位：
- 发送 IPI 时置位 SSIP
- 处理完成后清除 SSIP

代码位置参考
- `os/src/arch/riscv/ipi.rs`
- `os/src/arch/riscv/intr/mod.rs`

### 3.3 satp 与页表

内核使用 Sv39 模式，切换页表时写 satp 并执行全局 TLB 刷新。

代码位置参考
- `os/src/arch/riscv/mm/page_table.rs`

## 4. TrapFrame 与 CSR 关系

Trap 入口将 sepc、sstatus 等 CSR 保存到 TrapFrame，返回时恢复。sscratch 始终指向当前任务的 TrapFrame。

代码位置参考
- `os/src/arch/riscv/trap/trap_entry.S`
- `os/src/arch/riscv/trap/trap_frame.rs`
- `os/src/arch/riscv/kernel/cpu.rs`
