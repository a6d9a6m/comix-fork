# CCYOS 设计分析

## 基础部分
本段是 CCYOS 的设计分析文档第一部分，聚焦当前仓库的基础部分，表明对开发基础的认识。

### 1. 启动与架构层（RISC-V）

系统的启动流程要点如下（主核路径）：
- 汇编入口由 `os/src/arch/riscv/boot/entry.S` 提供，完成最小上下文与栈准备后跳入 Rust 入口。
- `os/src/arch/riscv/boot/mod.rs` 中的 `main()` 先清理 BSS、运行早期测试与打印，再初始化内存子系统并激活内核地址空间。
- 进入 `trap::init_boot_trap()`、`platform::init()`、`time::init()`，完成陷入入口与平台设备的基础初始化。
- 根据 `NUM_CPU` 启动从核，随后初始化定时器并创建 CPU0 idle 任务。
- 通过 `rest_init()` 构造第一个内核任务，并在 `init()` 中创建 kthreadd、初始化文件系统与网络配置，最后进入用户态 init（busybox 脚本）。

目前的项目架构采用“arch 层只处理指令级细节，内核层统一语义”的分层方式：
- `os/src/arch/riscv` 管理启动、陷入入口、寄存器保存与中断控制等架构相关逻辑。
- `os/src/kernel` 负责任务与调度、系统调用分发、时间管理等核心语义。
- `os/src/memory` 负责页表/帧分配/地址空间抽象，向内核提供统一的内存接口。

核心目录关系（以依赖方向理解）：
- `os/src/arch` 为最底层实现细节，向上提供 trap/interrupt/平台初始化能力。
- `os/src/kernel`、`os/src/memory`、`os/src/virtual_fs`、`os/src/filesystem` 构成主体逻辑层。
- `os/src/device` 和 `os/src/network` 作为平台功能扩展，依赖内核与内存/VFS 能力提供具体服务。

### 2. 陷入、异常与系统调用

- 统一的 trap 入口由架构层提供，异常与中断被规整为可分发的事件流，再交由内核处理。比如：
- 系统调用分发逻辑位于 `os/src/kernel/syscall`，内核实现以“ABI 子集”方式逐步扩展，便于按需推进。与arch的交互关系是，
- 设计意图是建立“低层可移植、高层可演进”的分层：trap 语义在架构层收敛，系统调用语义在内核层稳定。（这里需要举例）

### 3. 内存管理与地址空间

- 物理页帧分配与全局堆已经实现；虚拟内存采用 SV39，地址空间抽象在 `os/src/memory/memory_space/memory_space.rs` 等模块中落地。
- 用户态 ELF 加载、用户栈构建与地址空间切换配合执行，为 `execve` 等系统调用提供了基础。
- 当前重点是“可用性与结构清晰”，后续可针对 TLB 刷新策略、页表分配策略与内存回收路径做深入分析。

### 4. 任务、调度与时间

- 任务与调度位于 `os/src/kernel/task_control` 与 `os/src/kernel/scheduler`，实现了基础多任务骨架与简单调度器。
- `os/src/kernel/timer.rs` 与 `os/src/kernel/time.rs` 为时间片/定时机制提供支撑，保证内核可进行时间驱动的调度与超时逻辑。
- 设计侧重“可解释的流程与最小可用”，为后续引入更复杂的调度策略留下扩展点。

### 5. IPC 与信号

- IPC 框架位于 `os/src/interprocess`，包含 pipe、message、shared_memory 与 signal 的基础结构。
- 信号机制在接口层已接入（如 `os/src/interprocess/signal.rs`），但行为语义与边界条件仍属于可演进的核心区域。

### 6. VFS 与具体文件系统

- 虚拟文件系统位于 `os/src/virtual_fs`，提供路径解析、挂载、FD 表与文件锁等核心能力。
- 具体文件系统位于 `os/src/filesystem`，包含 simple_fs/smfs、tmpfs、procfs、sysfs；ext4 通过 VirtIO-Block 读写（ext4_rs 适配）形成真实块设备读写路径。
- 目前的设计注重“多层结构 + 可替换后端”，为实验提供对比平台。

### 7. 设备与驱动

- 设备与驱动位于 `os/src/device`，涵盖 VirtIO MMIO 框架、RAMDisk、VirtIO-Block、UART console、RTC 以及基础网卡适配骨架。
- 设备树读取与驱动注册形成了设备管理的统一入口，减少平台迁移成本。

### 8. 构建、运行与用户态打包

- 构建入口为顶层 `Makefile` 与 `os/` 内的构建脚本，运行入口包含 `os/qemu-run.sh`。
- `user/` 下的程序在构建时被编译并打包进 rootfs 镜像，与 `data/` 共同形成可运行的用户态环境。
- 这一流程保证“内核 + 用户态 + 文件系统”的一体化验证，适合迭代开发。


