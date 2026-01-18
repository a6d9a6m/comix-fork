//! ProcFS 符号链接测试

use super::*;
use crate::{kassert, test_case};

fn has_current_task() -> bool {
    crate::kernel::try_current_task().is_some()
}

test_case!(test_procfs_self_symlink_exists, {
    if !has_current_task() {
        // 测试环境可能没有 current_task，跳过该用例
        return;
    }

    let procfs = create_test_procfs_with_tree().unwrap();
    let root = procfs.root_inode();

    let self_link = root.lookup("self");
    kassert!(self_link.is_ok());

    let inode = self_link.unwrap();
    let metadata = inode.metadata().unwrap();
    kassert!(metadata.inode_type == InodeType::Symlink);
});

// TODO: 此测试需要 current_task，需要完整的内核上下文
// test_case!(test_procfs_self_symlink_read, {
//     let procfs = create_test_procfs_with_tree().unwrap();
//     let root = procfs.root_inode();
//     let self_link = root.lookup("self").unwrap();
//
//     // 读取符号链接目标
//     let target = self_link.readlink();
//     kassert!(target.is_ok());
//
//     let target_path = target.unwrap();
//     // 应该指向一个 PID（数字字符串）
//     kassert!(target_path.len() > 0);
// });
//
// test_case!(test_procfs_self_symlink_dynamic, {
//     // /proc/self 是动态符号链接，每次读取可能返回不同的PID
//     let procfs = create_test_procfs_with_tree().unwrap();
//     let root = procfs.root_inode();
//     let self_link = root.lookup("self").unwrap();
//
//     let target1 = self_link.readlink().unwrap();
//     let target2 = self_link.readlink().unwrap();
//
//     // 在单线程测试中应该指向同一个进程
//     kassert!(target1 == target2);
// });

test_case!(test_procfs_symlink_metadata, {
    if !has_current_task() {
        return;
    }

    let procfs = create_test_procfs_with_tree().unwrap();
    let root = procfs.root_inode();
    let self_link = root.lookup("self").unwrap();

    let metadata = self_link.metadata().unwrap();
    kassert!(metadata.inode_type == InodeType::Symlink);
    kassert!(metadata.nlinks == 1);
});

test_case!(test_procfs_symlink_not_writable, {
    if !has_current_task() {
        return;
    }

    let procfs = create_test_procfs_with_tree().unwrap();
    let root = procfs.root_inode();
    let self_link = root.lookup("self").unwrap();

    // 符号链接不应该支持写入
    let result = self_link.write_at(0, b"test");
    kassert!(result.is_err());
});

test_case!(test_procfs_symlink_not_readable_as_file, {
    if !has_current_task() {
        return;
    }

    let procfs = create_test_procfs_with_tree().unwrap();
    let root = procfs.root_inode();
    let self_link = root.lookup("self").unwrap();

    // 符号链接不应该支持 read_at（应使用 read_link）
    let mut buf = [0u8; 10];
    let result = self_link.read_at(0, &mut buf);
    kassert!(result.is_err());
});
