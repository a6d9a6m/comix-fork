# 等待队列 (WaitQueue)

`WaitQueue` 用于阻塞与唤醒任务，配合调度器实现睡眠等待。

源码位置: `os/src/kernel/scheduler/wait_queue.rs`

## 1. 数据结构

- `tasks: TaskQueue`
- `lock: RawSpinLock`

内部通过自旋锁保护队列，避免并发访问竞争。

## 2. 核心接口

- `sleep(task)`: 入队并调用 `sleep_task_with_block()`
- `wake_up(task)`: 移出指定任务并唤醒
- `wake_up_one()`: 唤醒队首任务
- `wake_up_all()`: 批量唤醒所有任务
- `sleep_if(task, check_fn)`: 原子检查条件并睡眠，用于避免 lost wakeup

## 3. 典型流程

1. 资源不可用时调用 `sleep()`
2. 任务状态切为 `Interruptible` 或 `Uninterruptible`
3. 资源可用时调用 `wake_up_*()`
4. 调度器将任务重新放入运行队列

## 4. 注意事项

- `sleep()` 会持锁入队后立即释放锁，避免在持锁时调度。
- `wake_up_*()` 在锁外执行唤醒，减少临界区长度。

