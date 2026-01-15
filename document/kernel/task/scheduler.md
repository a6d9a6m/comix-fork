# 任务调度

本页描述调度器接口、RRScheduler 实现与调度触发路径。

## 1. 调度器接口

接口定义在 `os/src/kernel/scheduler/mod.rs`:

- `add_task(task)`: 入队
- `next_task() -> Option<SwitchPlan>`: 选择下一个任务并准备切换
- `sleep_task(task, receive_signal)`: 置为阻塞并移出队列
- `wake_up(task)`: 恢复 Running 并入队
- `exit_task(task)`: 置为 Zombie 并移除

`SwitchPlan` 包含 `old/new Context` 指针，供底层 `switch()` 使用。

## 2. 每 CPU 调度器

- `SCHEDULERS`: 固定大小的 per-CPU 调度器数组
- `current_scheduler()` 根据 `cpu_id()` 获取当前调度器
- `pick_cpu()` 简单轮询选择目标 CPU

多核唤醒时会发送 IPI:
- `wake_up_with_block()` 在跨核唤醒时调用 `send_reschedule_ipi()`

## 3. RRScheduler (轮转)

实现位于 `os/src/kernel/scheduler/rr_scheduler.rs`:

- 运行队列: `TaskQueue`
- 时间片: `DEFAULT_TIME_SLICE = 1`
- `next_task()` 负责:
  - 从队列取任务
  - 更新 `current_cpu().switch_task()`
  - 将旧任务放回队尾 (若仍 Running)
  - 切到 idle 任务 (队列为空且当前任务非 Running)

## 4. 调度触发点

- 主动让出: `yield_task()`
- 阻塞/唤醒: `sleep_task_with_block()` / `wake_up_with_block()`
- 时钟中断: trap 处理中调用 `schedule()` (见 `os/src/arch/riscv/trap/trap_handler.rs`)

`schedule()` 会:
1. 关闭中断
2. 运行 `next_task()`
3. 调用 `switch(old, new)`
4. 恢复中断

## 5. 关键源码

- `os/src/kernel/scheduler/mod.rs`
- `os/src/kernel/scheduler/rr_scheduler.rs`
- `os/src/arch/riscv/kernel/switch.S`

