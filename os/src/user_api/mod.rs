//! 与用户空间共用定义和声明
//!
//! 包含常量、类型和函数声明，确保内核和用户空间的一致性

#![allow(dead_code)]
pub mod credentials;
pub mod errno;
pub mod fast_userspace_mutex;
pub mod file_control;
pub mod filesystem;
pub mod io_control;
pub mod iovec;
pub mod logging;
pub mod memory;
pub mod reboot;
pub mod resource;
pub mod sched;
pub mod select;
pub mod signal;
pub mod socket;
pub mod sysinfo;
pub mod time;
pub mod types;
pub mod uts_namespace;
pub mod wait;
