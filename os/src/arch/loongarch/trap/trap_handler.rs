//! LoongArch64 陷阱处理实现（与 RISC-V 路径一致的接口）。

use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use crate::arch::constant::SUPERVISOR_EXTERNAL;
use crate::arch::constant::{
    CSR_BADI, CSR_BADV, CSR_CRMD_PLV_MASK, CSR_EENTRY, CSR_ESTAT_IS_MASK, CSR_TLBRENT,
};
use crate::arch::syscall::dispatch_syscall;
use crate::arch::timer::{
    TIMER_TICKS, ack_timer_interrupt, clock_freq, get_time, set_next_trigger,
};
use crate::arch::trap::restore;
use crate::device::IRQ_MANAGER;
use crate::earlyprintln;
use crate::interprocess::check_signal;
use crate::kernel::{TIMER, TIMER_QUEUE, schedule, send_signal_process, wake_up_with_block};
use crate::memory::address::{PageNum, UsizeConvert};
use crate::memory::page_table::PageTableInner;

use super::TrapFrame;

/// 仅在单核环境下使用的默认 TrapFrame；后续可由调度器替换为 per-CPU/任务帧
#[unsafe(no_mangle)]
pub static mut BOOT_TRAP_FRAME: TrapFrame = TrapFrame::empty();

static FIRST_TRAP_LOGGED: AtomicBool = AtomicBool::new(false);
static FIRST_USER_TIMER_LOGGED: AtomicBool = AtomicBool::new(false);
static USER_SYSCALL_LOG_BUDGET: AtomicUsize = AtomicUsize::new(16);

const ECODE_SYSCALL: usize = 0xb; // LoongArch syscall 异常码
const TIMER_INT_BIT: usize = 1 << 11; // ESTAT.IS 中的本地定时器位

unsafe extern "C" {
    unsafe fn __restore(tf: &TrapFrame);
    unsafe fn trap_entry();
    unsafe fn tlb_refill_entry();
}

/// 汇编入口调用的陷阱处理函数。
#[unsafe(no_mangle)]
pub extern "C" fn trap_handler(trap_frame: &mut TrapFrame) {
    let prmd = trap_frame.prmd;
    let estat = trap_frame.estat;
    let era = trap_frame.era;

    if !FIRST_TRAP_LOGGED.swap(true, Ordering::Relaxed) {
        let badv: usize;
        let badi: usize;
        let pgdl: usize;
        let pgdh: usize;
        unsafe {
            core::arch::asm!("csrrd {0}, {csr}", out(reg) badv, csr = const CSR_BADV, options(nostack, preserves_flags));
            core::arch::asm!("csrrd {0}, {csr}", out(reg) badi, csr = const CSR_BADI, options(nostack, preserves_flags));
            core::arch::asm!("csrrd {0}, 0x19", out(reg) pgdl, options(nostack, preserves_flags));
            core::arch::asm!("csrrd {0}, 0x1a", out(reg) pgdh, options(nostack, preserves_flags));
        }
        crate::pr_debug!(
            "[trap_handler] first trap: estat={:#x}, era={:#x}, prmd={:#x}, crmd={:#x}, badv={:#x}, badi={:#x}, pgdl={:#x}, pgdh={:#x}",
            estat,
            era,
            prmd,
            trap_frame.crmd,
            badv,
            badi,
            pgdl,
            pgdh
        );
    }

    if (prmd & CSR_CRMD_PLV_MASK) != 0 {
        user_trap(estat, era, trap_frame);
    } else {
        kernel_trap(estat, era, trap_frame);
    }

    check_signal();

    // 恢复“当前任务”的陷阱帧；若没有当前任务，回退到入口参数。
    let tf_ptr = crate::kernel::try_current_task()
        .map(|t| t.lock().trap_frame_ptr.load(Ordering::SeqCst) as usize)
        .unwrap_or(trap_frame as *mut _ as usize);
    // 安全性：指针来源于当前任务保存的 trap_frame_ptr 或回退到入口参数。
    unsafe { restore(&*(tf_ptr as *const TrapFrame)) };
}

/// 安装启动阶段的陷阱入口
pub(super) fn install_boot_trap() {
    install_trap_entry();
}

/// 安装运行期的陷阱入口
pub(super) fn install_runtime_trap() {
    install_trap_entry();
}

fn install_trap_entry() {
    // 将 TrapFrame 指针写入 KScratch0，并设置 EENTRY 指向 trap_entry
    unsafe {
        // 设置内核栈指针用于用户态陷阱的栈切换
        let sp: usize;
        core::arch::asm!("addi.d {0}, $sp, 0", out(reg) sp, options(nostack, preserves_flags));
        BOOT_TRAP_FRAME.kernel_sp = sp;
        BOOT_TRAP_FRAME.cpu_ptr = crate::kernel::current_cpu() as *const _ as usize;

        // KScratch0 <- TrapFrame 指针
        core::arch::asm!(
            "csrwr {0}, 0x30",
            in(reg) (&raw mut BOOT_TRAP_FRAME as *mut TrapFrame as usize),
            options(nostack, preserves_flags)
        );
        // EENTRY <- trap_entry（注意 CSR 编号为 0xc）
        core::arch::asm!(
            "csrwr {val}, {csr}",
            val = in(reg) trap_entry as usize,
            csr = const CSR_EENTRY,
            options(nostack, preserves_flags)
        );
        // TLB refill 入口使用独立处理，进行页表遍历与 tlbfill
        // TLBRENT 必须使用物理地址，因为 refill 时 CPU 处于直接地址翻译模式
        let tlbr_entry_paddr =
            unsafe { crate::arch::memory::vaddr_to_paddr(tlb_refill_entry as usize) } & !0xfff;
        core::arch::asm!(
            "csrwr {val}, {csr}",
            val = in(reg) tlbr_entry_paddr,
            csr = const CSR_TLBRENT,
            options(nostack, preserves_flags)
        );

        // 设置 TLBIDX.PS = 12 (4KB 页)
        // TLBIDX 的 PS 字段在 bits [29:24]
        core::arch::asm!(
            "csrrd $t0, 0x10",
            "li.w $t1, 12",
            "bstrins.d $t0, $t1, 29, 24",
            "csrwr $t0, 0x10",
            out("$t0") _,
            out("$t1") _,
            options(nostack)
        );

        // 设置 TLBREHI.PS = 12 (4KB 页)
        // TLBREHI 的 PS 字段在 bits [5:0]
        core::arch::asm!(
            "csrrd $t0, 0x8e",
            "li.w $t1, 12",
            "bstrins.d $t0, $t1, 5, 0",
            "csrwr $t0, 0x8e",
            out("$t0") _,
            out("$t1") _,
            options(nostack)
        );

        let tlbrent: usize;
        core::arch::asm!(
            "csrrd {0}, {csr}",
            out(reg) tlbrent,
            csr = const CSR_TLBRENT,
            options(nostack, preserves_flags)
        );
        crate::pr_info!(
            "[trap_init] tlbrent={:#x}, tlbr_paddr={:#x}",
            tlbrent,
            tlbr_entry_paddr
        );
    }
}

fn user_trap(estat: usize, era: usize, trap_frame: &mut TrapFrame) {
    crate::pr_debug!("[user_trap] estat={:#x} era={:#x}", estat, era);

    // 优先处理中断
    if estat & CSR_ESTAT_IS_MASK != 0 {
        if (estat & TIMER_INT_BIT) != 0 && !FIRST_USER_TIMER_LOGGED.swap(true, Ordering::Relaxed) {
            crate::pr_debug!("[user_trap] first user timer interrupt, era={:#x}", era);
        }
        handle_interrupt(estat);
        return;
    }

    // 非中断路径按异常码分发
    let ecode = (estat >> 16) & 0x3f;
    match ecode {
        ECODE_SYSCALL => {
            let syscall_id = trap_frame.syscall_id();
            let log_now = USER_SYSCALL_LOG_BUDGET
                .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_sub(1))
                .is_ok();
            if log_now {
                crate::pr_info!("[user_trap] syscall id={}, era={:#x}", syscall_id, era);
            } else {
                crate::pr_debug!("[user_trap] syscall id={}, era={:#x}", syscall_id, era);
            }
            trap_frame.era = era.wrapping_add(4);
            dispatch_syscall(trap_frame);
        }
        _ => {
            user_panic(estat, era, trap_frame);
        }
    }
}

fn kernel_trap(estat: usize, era: usize, tf: &TrapFrame) {
    // 内核态中断优先处理，避免把中断当作异常
    if estat & CSR_ESTAT_IS_MASK != 0 {
        handle_interrupt(estat);
        return;
    }

    let ecode = (estat >> 16) & 0x3f;
    let badv: usize;
    let badi: usize;
    unsafe {
        core::arch::asm!("csrrd {0}, {csr}", out(reg) badv, csr = const CSR_BADV, options(nostack, preserves_flags));
        core::arch::asm!("csrrd {0}, {csr}", out(reg) badi, csr = const CSR_BADI, options(nostack, preserves_flags));
    }
    panic!(
        "Unexpected trap in kernel: ecode={:#x}, estat={:#x}, era={:#x}, badv={:#x}, badi={:#x}, crmd={:#x}, prmd={:#x}, a0={:#x}, a1={:#x}",
        ecode,
        estat,
        era,
        badv,
        badi,
        tf.crmd,
        tf.prmd,
        tf.regs[4], // a0
        tf.regs[5], // a1
    );
}

fn handle_interrupt(estat: usize) {
    let pending = estat & CSR_ESTAT_IS_MASK;
    // 本地定时器中断
    if pending & TIMER_INT_BIT != 0 {
        ack_timer_interrupt();
        set_next_trigger();
        check_timer();
    }
    // 其他中断来源（外设/软件中断等）
    if pending & !TIMER_INT_BIT != 0 {
        check_device();
    }
}

/// 处理用户态异常：打印诊断信息并终止任务
fn user_panic(estat: usize, era: usize, trap_frame: &TrapFrame) {
    let badv: usize;
    let badi: usize;
    unsafe {
        core::arch::asm!("csrrd {0}, {csr}", out(reg) badv, csr = const CSR_BADV, options(nostack, preserves_flags));
        core::arch::asm!("csrrd {0}, {csr}", out(reg) badi, csr = const CSR_BADI, options(nostack, preserves_flags));
    }
    earlyprintln!("\n===============================================");
    earlyprintln!("   UNEXPECTED TRAP IN USER MODE (PLV>0)");
    earlyprintln!("===============================================");
    earlyprintln!("estat: {:#x}", estat);
    earlyprintln!("era  : {:#x}", era);
    earlyprintln!("badv : {:#x}", badv);
    earlyprintln!("badi : {:#x}", badi);
    let ecode = (estat >> 16) & 0x3f;
    let esubcode = (estat >> 22) & 0x1ff;
    earlyprintln!("ecode: {:#x}", ecode);
    earlyprintln!("esub : {:#x}", esubcode);
    earlyprintln!("regs : {:#x?}", trap_frame.regs);
    {
        use crate::memory::address::{Vaddr, Vpn};
        let tramp = crate::config::USER_SIGRETURN_TRAMPOLINE;
        if badv >= tramp && badv < tramp + crate::config::PAGE_SIZE {
            earlyprintln!("提示 : 故障地址位于用户态 sigreturn trampoline 页");
        }
        if let Some(space) = crate::kernel::current_task().lock().memory_space.clone() {
            let space = space.lock();
            let vaddr = Vaddr::from_usize(badv);
            let vpn = Vpn::from_addr_floor(vaddr);
            match space.page_table().walk(vpn) {
                Ok((ppn, _, flags)) => {
                    earlyprintln!(
                        "pte  : vpn={:#x} -> ppn={:#x}, flags={:?}",
                        vpn.0,
                        ppn.0,
                        flags
                    );
                }
                Err(e) => {
                    earlyprintln!("pte  : walk failed: {:?}", e);
                }
            }
        }
    }
    let sig = match ecode {
        // 地址/权限/页相关异常 → SIGSEGV
        0x1 | 0x2 | 0x3 | 0x4 | 0x5 | 0x6 | 0x7 => crate::user_api::signal::NUM_SIGSEGV,
        0x8 | 0x9 | 0xA => crate::user_api::signal::NUM_SIGSEGV,
        // 断点 → SIGTRAP
        0xC => crate::user_api::signal::NUM_SIGTRAP,
        // 非法/特权/禁用指令 → SIGILL
        0xD | 0xE | 0xF | 0x10 | 0x11 | 0x14 | 0x15 => crate::user_api::signal::NUM_SIGILL,
        // 浮点异常 → SIGFPE
        0x12 => crate::user_api::signal::NUM_SIGFPE,
        _ => {
            if badi != 0 || badv == era {
                crate::user_api::signal::NUM_SIGILL
            } else {
                crate::user_api::signal::NUM_SIGSEGV
            }
        }
    };
    crate::kernel::terminate_task(128 + sig);
}

#[allow(dead_code)]
fn check_device() {
    IRQ_MANAGER
        .read()
        .try_handle_interrupt(Some(SUPERVISOR_EXTERNAL));
}

/// 处理时钟中断：更新时间片/定时器队列，并触发调度
fn check_timer() {
    let _ticks = TIMER_TICKS.fetch_add(1, Ordering::Relaxed);
    while let Some(task) = TIMER_QUEUE.lock().pop_due_task(get_time()) {
        wake_up_with_block(task);
    }
    while let Some(entry) = TIMER.lock().pop_due_entry(get_time()) {
        send_signal_process(&entry.task, entry.sig);
        if !entry.it_interval.is_zero() {
            let next_trigger = get_time() + entry.it_interval.into_freq(clock_freq());
            TIMER.lock().push(next_trigger, entry);
        }
    }
    let should_preempt = {
        let mut sched = crate::kernel::current_scheduler().lock();
        sched.update_time_slice() && !sched.is_empty()
    };
    if should_preempt {
        schedule();
    }
}

/// 恢复陷阱前的上下文（由汇编实现）
pub(super) fn restore_context(trap_frame: &TrapFrame) {
    unsafe { __restore(trap_frame) }
}

// 信号返回跳板由汇编提供（sigreturn.S）
