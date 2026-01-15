# RISC-V 用户栈布局与初始化

本页描述 RISC-V 用户进程在 execve/kexec 时的栈布局与构造流程，内容以代码实现为准。

## 1. 入口与数据流

- 栈构造入口: `os/src/arch/riscv/kernel/task.rs` 的 `setup_stack_layout()`
- 典型调用路径: `os/src/kernel/task/task_struct.rs`
- TrapFrame 设置: `os/src/arch/riscv/trap/trap_frame.rs` 的 `set_exec_trap_frame()`

`setup_stack_layout()` 返回 `(sp, argc, argv_ptr, envp_ptr)`，随后写入 TrapFrame:
- `a0 = argc`
- `a1 = argv_ptr`
- `a2 = envp_ptr`
- `sp = sp`

## 2. 栈区范围与对齐

- 用户栈顶: `USER_STACK_TOP` (见 `os/src/config.rs`)
- 栈大小: `USER_STACK_SIZE`
- 栈向低地址增长
- 16 字节对齐: `STACK_ALIGN_MASK = 0xF` (见 `os/src/arch/riscv/constant.rs`)

`setup_stack_layout()` 会在构造指针区块前做 16 字节对齐，并确保最终 sp 满足 ABI 对齐要求。

## 3. 实际布局 (高地址 -> 低地址)

```
[ stack_top ]
  ...
  envp 字符串 (以 NUL 结尾)
  argv 字符串 (以 NUL 结尾)
  AT_RANDOM 16 字节
  "riscv64\0" 平台字符串
  padding (16-byte align)
  auxv: (type, val) 对数组
  envp NULL
  envp[0..n-1]
  argv NULL
  argv[0..n-1]
  argc
[ sp ]
```

关键说明
- 字符串区先写 envp 再写 argv，均为反向写入。
- auxv 放在 envp NULL 之上，指针数组之下。
- 目前 `AT_RANDOM` 使用固定常量字节，不是随机源。

## 4. auxv 条目

`setup_stack_layout()` 写入的 auxv 条目如下 (type, value):

- `AT_PHDR` (3): `phdr_addr`
- `AT_PHENT` (4): `phent`
- `AT_PHNUM` (5): `phnum`
- `AT_PAGESZ` (6): 4096
- `AT_BASE` (7): `at_base`
- `AT_ENTRY` (9): `at_entry`
- `AT_UID` (11): 0
- `AT_EUID` (12): 0
- `AT_GID` (13): 0
- `AT_EGID` (14): 0
- `AT_PLATFORM` (15): "riscv64\0" 指针
- `AT_HWCAP` (16): 0
- `AT_CLKTCK` (17): 100
- `AT_SECURE` (23): 0
- `AT_RANDOM` (25): 16 字节指针
- `AT_EXECFN` (31): argv[0] 指针 (不存在则为 0)
- `AT_NULL` (0): 结束标记

## 5. 内核写入用户栈的约束

`setup_stack_layout()` 直接向用户地址写内存，因此需要:
- 目标页已映射到用户空间
- 通过 `sstatus::set_sum()` 允许内核访问 U 页

函数内部会 `set_sum()` 并在末尾 `clear_sum()`。调用者无需重复处理，但必须保证映射已就绪。

## 6. 失败场景与排查

常见问题
- sp 未对齐: 用户态 ABI 崩溃或 libc 异常
- envp/argv NULL 终止缺失: 用户程序遍历越界
- USER_STACK_TOP 与 trampoline 区间冲突: 参见 `USER_SIGRETURN_TRAMPOLINE` 约束

建议排查点
- `os/src/arch/riscv/kernel/task.rs`
- `os/src/kernel/task/task_struct.rs`
- `os/src/arch/riscv/trap/trap_frame.rs`

