# RISC-V IPI (核间中断)

本页描述 RISC-V IPI 在 Comix 中的实现与使用方式。

## 1. 机制概览

- IPI 基于 SBI 软件中断发送 (`sbi::send_ipi`)
- 每核使用一个 `AtomicU32` 记录待处理 IPI 类型
- 接收端在软件中断处理路径中调用 `handle_ipi()`

代码位置: `os/src/arch/riscv/ipi.rs`

## 2. IPI 类型

`IpiType` 采用位标志，可组合:

- `Reschedule` (1 << 0): 触发调度
- `TlbFlush` (1 << 1): 刷新 TLB
- `Stop` (1 << 2): 停止 CPU

`IPI_PENDING` 为 `MAX_CPU_COUNT` 大小的原子数组。

## 3. 发送路径

### 3.1 单核发送

`send_ipi(target_cpu, ipi_type)`:
- 校验 `target_cpu < NUM_CPU`
- `fetch_or` 设置目标 CPU 的待处理标志
- 通过 `sbi::send_ipi` 触发软件中断

### 3.2 批量发送

`send_ipi_many(hart_mask, ipi_type)`:
- 按位掩码设置多个 CPU 的 `IPI_PENDING`
- 只调用一次 `sbi::send_ipi`

### 3.3 辅助接口

- `send_reschedule_ipi(cpu)`
- `send_tlb_flush_ipi_all()` (广播到除当前 CPU 外的所有 CPU)

## 4. 接收路径

`handle_ipi()` 执行以下步骤:

1. 清除 `sip.SSIP` (软件中断挂起位)
2. 读取并清空当前 CPU 的 `IPI_PENDING`
3. 根据标志位执行动作:
   - Reschedule: 仅设置标志，调度在 trap 返回路径中触发
   - TlbFlush: 执行 `sfence.vma`
   - Stop: 进入 `wfi` 循环

调用入口在 trap 处理:
- `os/src/arch/riscv/trap/trap_handler.rs`
- 软件中断类型 `Trap::Interrupt(1)`

调度触发条件
- 用户态 trap: 运行队列非空时调度
- 内核态 trap: 运行队列非空时调度

## 5. 与 TLB Shootdown 的关系

RISC-V 页表实现会在必要时调用 `send_tlb_flush_ipi_all()`，用于多核 TLB 同步。

代码位置
- `os/src/arch/riscv/mm/page_table.rs`
- 支持批处理上下文以减少 IPI 数量 (见 `TlbBatchContext`)

## 6. 约束与注意事项

- IPI 处理在中断上下文执行，禁止内存分配与睡眠锁
- `Stop` IPI 会进入无限 `wfi`，用于关机/停机场景
- `send_ipi` 会在目标 CPU 上产生软件中断，需要确保 `trap::init()` 已启用软件中断

## 7. 关键代码位置

- `os/src/arch/riscv/ipi.rs`
- `os/src/arch/riscv/lib/sbi.rs`
- `os/src/arch/riscv/intr/mod.rs`
- `os/src/arch/riscv/trap/mod.rs`
- `os/src/arch/riscv/trap/trap_handler.rs`

