use alloc::vec::Vec;

use crate::filesystem::proc::inode::ContentGenerator;
use crate::virtual_fs::FsError;

pub struct CpuinfoGenerator;

impl ContentGenerator for CpuinfoGenerator {
    fn generate(&self) -> Result<Vec<u8>, FsError> {
        Ok(crate::arch::info::proc_cpuinfo())
    }
}
