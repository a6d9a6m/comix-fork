# 任务管理概述

本目录描述内核任务管理的实现。Comix 以 `Task` 统一进程与线程概念，线程通过共享资源实现轻量化，任务切换由调度器驱动。

## 范围

- 任务数据结构与生命周期
- 调度器与上下文切换
- 等待队列与阻塞/唤醒
- 任务地址空间与 execve 相关流程

## 导航

- `document/kernel/task/task.md`：Task 结构与生命周期
- `document/kernel/task/context.md`：任务上下文与切换
- `document/kernel/task/scheduler.md`：调度器实现与调度路径
- `document/kernel/task/wait_queue.md`：等待队列与阻塞唤醒
- `document/kernel/task/memory_space.md`：任务地址空间与 execve

## 关键源码

- `os/src/kernel/task/task_struct.rs`
- `os/src/kernel/task/task_state.rs`
- `os/src/kernel/scheduler/mod.rs`
- `os/src/kernel/scheduler/rr_scheduler.rs`
- `os/src/kernel/scheduler/wait_queue.rs`

