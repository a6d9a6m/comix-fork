# 任务结构与生命周期

本页描述 `Task` 的核心字段、进程/线程关系与生命周期。内容以当前实现为准。

## 1. 任务模型

- `Task` 统一表示进程与线程。
- 进程满足 `pid == tid`，线程满足 `pid != tid`。
- 线程共享资源（文件、信号、地址空间等）通过 `Arc` 持有。
- 每个任务都有独立内核栈与寄存器上下文。

源码位置: `os/src/kernel/task/task_struct.rs`

## 2. 关键字段分组

### 2.1 身份与层级

- `tid`, `pid`, `ppid`, `pgid`
- `children`: 子任务列表
- `wait_child`: 等待子任务退出的等待队列

### 2.2 调度与执行

- `context`: 任务切换所需最小上下文
- `trap_frame_ptr`: TrapFrame 指针
- `state`: `TaskState` (Running/Stopped/Interruptible/Uninterruptible/Zombie)
- `on_cpu`: 当前运行 CPU，`cpu_affinity` 亲和性掩码

### 2.3 地址空间与栈

- `memory_space: Option<MemorySpace>`
  - 内核线程为 `None`
  - 用户任务持有地址空间
- `kstack_base`: 内核栈基址

### 2.4 资源与命名空间

- `fd_table`: 文件描述符表
- `fs`: 文件系统相关信息 (cwd/root)
- `uts_namespace`, `rlimit`, `credential`, `umask`

### 2.5 信号与退出

- `blocked`, `pending`, `shared_pending`, `signal_handlers`, `signal_stack`
- `exit_code`, `exit_signal`

## 3. 任务创建

- 内核线程: `Task::ktask_create()`
- 用户任务: `Task::utask_create()`

创建流程通常包括:
1. 分配内核栈与 TrapFrame
2. 初始化 `Context`，`ra` 指向 `forkret`
3. 将任务加入调度器运行队列

相关入口:
- `os/src/kernel/task/ktask.rs`
- `os/src/kernel/task/task_manager.rs`

## 4. 执行与切换

- 任务首次被调度会进入 `forkret`，由 `restore()` 恢复 TrapFrame。
- 上下文切换由 `schedule()` 驱动，底层使用 `__switch`。

相关代码:
- `os/src/kernel/task/mod.rs` (`forkret`)
- `os/src/arch/riscv/kernel/switch.S`

## 5. execve 与地址空间替换

`Task::execve()` 会:
1. 切换到新的 `MemorySpace`
2. 复制/关闭 `FD_CLOEXEC` 文件
3. 构造用户栈 (argv/envp/auxv)
4. 更新 TrapFrame 并写入用户入口地址

相关代码:
- `os/src/kernel/syscall/task.rs`
- `os/src/kernel/task/exec_loader.rs`
- `os/src/kernel/task/task_struct.rs`

## 6. 退出与回收

- `terminate_task()` 处理致命异常与线程退出。
- 进程退出由 `exit_process()` 统一处理，回收资源并通知父任务。
- `TaskState` 变为 `Zombie` 后由父进程 `wait4` 回收。

相关代码:
- `os/src/kernel/task/process.rs`
- `os/src/kernel/task/task_manager.rs`

