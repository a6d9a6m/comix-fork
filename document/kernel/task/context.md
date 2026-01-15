# 任务上下文 (Context)

`Context` 用于任务调度切换时保存最小寄存器集合，属于非中断驱动的切换路径。

## 1. 数据结构

定义位置: `os/src/arch/riscv/kernel/context.rs`

```rust
pub struct Context {
    pub ra: usize,
    pub sp: usize,
    s: [usize; 12], // s0..s11
}
```

保存范围符合 RISC-V 调用约定：仅保存被调用者保存寄存器。

## 2. 使用场景

- `schedule()` 触发任务切换
- `__switch(old, new)` 保存/恢复 `Context`

底层汇编: `os/src/arch/riscv/kernel/switch.S`

## 3. 与 TrapFrame 的关系

- `Context` 只用于普通调度路径。
- 中断/系统调用路径使用 `TrapFrame` 保存完整上下文。

相关文档: `document/kernel/trap/context.md`

## 4. 初始化

新任务创建时:
- `Context::zero_init()` 清零
- `set_init_context(forkret, kstack_base)` 让首次调度进入 `forkret`

相关代码: `os/src/kernel/task/task_struct.rs`

