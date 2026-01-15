# RISC-V 多核启动 (SMP Boot)

本页描述 RISC-V 在 Comix 中的多核启动流程，按当前代码实现整理。

## 1. 启动角色

- 主核: hart 0，负责系统初始化与启动从核。
- 从核: hart 1..N，完成各自的 trap/idle 初始化后进入 idle_loop。

最大核数由 `MAX_CPU_COUNT` 限制 (见 `os/src/config.rs`)。

## 2. 主核启动路径

入口在 `os/src/arch/riscv/boot/mod.rs` 的 `main()`:

1. 清 BSS
2. 内存初始化 `mm::init()`
3. 初始化 `CPUS` 并设置 `tp` 指向 CPU0
4. 激活内核地址空间 `current_cpu().switch_space()`
5. 初始化 boot trap / 平台 / 时钟
6. 启动从核 `boot_secondary_cpus()`
7. 初始化定时器 `timer::init()`
8. 创建 CPU0 的 idle 任务
9. `rest_init()` 进入调度

说明
- 中断在 `rest_init()` 的 `init()` 阶段打开 (设置好 sscratch 后)。
- 启动从核发生在定时器初始化之前，避免主核等待时被中断打断。

## 3. 从核启动路径

从核入口为 `secondary_sbi_entry` (见 `os/src/arch/riscv/boot/entry.S`)，最终进入 `secondary_start()`:

1. `trap::init_boot_trap()` 设置 boot trap 入口
2. 设置 `tp` 指向本核 `Cpu` 结构
3. 标记在线 `CPU_ONLINE_MASK.fetch_or()`
4. `trap::init()` 启用完整 trap 处理并打开软件中断
5. 创建 idle 任务并设置 `sscratch` 指向其 TrapFrame
6. 将 idle 任务设为当前任务，并记录为本核 idle
7. 切换到全局内核页表 (若可用)
8. 初始化定时器并启用中断
9. 进入 `idle_loop()`

从核不会进入正常调度队列，只在 idle_loop 中等待。

## 4. 启动同步与降级策略

同步变量: `CPU_ONLINE_MASK` (每位代表一个 CPU 是否在线)

`boot_secondary_cpus()` 的策略:
- 仅对 `hart_start` 成功的从核计入 `expected_mask`
- 若没有任何从核接受启动请求，降级为单核
- 使用基于时钟的 2 秒超时等待
- 等待结束后，将 `NUM_CPU` 更新为实际在线核数

这保证了在 SBI 不支持或启动失败时系统仍可运行。

## 5. tp/sscratch 与 TrapFrame 关系

- `tp` 在内核态指向 `Cpu` 结构，用于快速 Per-CPU 访问
- `sscratch` 指向当前任务的 TrapFrame，供 trap_entry 保存/恢复上下文

关键位置
- `os/src/arch/riscv/boot/mod.rs` (tp 设置, sscratch 更新)
- `os/src/arch/riscv/trap/trap_entry.S`

## 6. 关键代码位置

- `os/src/arch/riscv/boot/mod.rs`
- `os/src/arch/riscv/boot/entry.S`
- `os/src/kernel/cpu.rs`
- `os/src/sync/per_cpu.rs`
- `os/src/arch/riscv/trap/mod.rs`

