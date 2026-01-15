# 任务地址空间与 execve

本页描述任务与地址空间 (`MemorySpace`) 的关系，以及 execve 的地址空间切换流程。

## 1. Task 与 MemorySpace

- `Task.memory_space: Option<Arc<SpinLock<MemorySpace>>>`
- 内核线程为 `None`
- 用户任务持有独立地址空间

相关代码:
- `os/src/kernel/task/task_struct.rs`
- `os/src/mm/memory_space/memory_space.rs`

## 2. execve 流程

入口: `os/src/kernel/syscall/task.rs::execve()`

核心步骤:
1. 解析路径与参数 (argv/envp)
2. `do_execve_prepare()` 解析 ELF 并构建新的 `MemorySpace`
3. `do_execve_switch()` 切换地址空间并调用 `Task::execve()`
4. `Task::execve()` 构造用户栈并更新 TrapFrame

用户栈构造:
- RISC-V: `os/src/arch/riscv/kernel/task.rs::setup_stack_layout()`
- LoongArch: `os/src/arch/loongarch/kernel/task.rs::setup_stack_layout()`

## 3. TrapFrame 设置

`Task::execve()` 会:
- 清零 TrapFrame
- 写入用户入口 `sepc`/`era`
- 设置用户栈 `sp`
- 写入 `a0/a1/a2` (argc/argv/envp)

对应代码:
- `os/src/arch/riscv/trap/trap_frame.rs`
- `os/src/arch/loongarch/trap/trap_frame.rs`

## 4. 资源处理

execve 会复制 FD 表并关闭 `FD_CLOEXEC`:
- `FDTable::clone_table()`
- `FDTable::close_exec()`

此外，会更新 `exe_path` 用于 `/proc/[pid]/exe`。

