use core::ffi::c_void;

#[cfg(all(feature = "std", any(target_os = "linux", target_os = "android")))]
pub use std::io::Result;

#[cfg(all(not(feature = "std"), any(target_os = "linux", target_os = "android")))]
pub type Result<T> = core::result::Result<T, core::ffi::c_int>;

#[cfg(all(feature = "std", not(any(target_os = "linux", target_os = "android"))))]
pub use anyhow::Result;

#[cfg(all(
    not(feature = "std"),
    not(any(target_os = "linux", target_os = "android"))
))]
pub type Result<T> = core::result::Result<T, ()>;

#[cfg(all(
    target_arch = "aarch64",
    any(target_os = "linux", target_os = "android")
))]
mod details {

    use super::*;
    use libc::{EINVAL, ENOSYS, EPERM, syscall};

    #[cfg(feature = "std")]
    use std::io::Error;

    const MEMBARRIER_CMD_GLOBAL: libc::c_int = 1;
    const MEMBARRIER_CMD_PRIVATE_EXPEDITED_SYNC_CORE: libc::c_int = 32;
    const MEMBARRIER_CMD_REGISTER_PRIVATE_EXPEDITED_SYNC_CORE: libc::c_int = 64;

    /// See docs on [crate::pipeline_flush_mt] for a description of what this function is trying to do.
    #[inline]
    pub(crate) fn pipeline_flush_mt() -> Result<()> {
        // Ensure that no processor has fetched a stale instruction stream.
        //
        // On AArch64 we try to do this by executing a "broadcast" `ISB` which is not something
        // that the architecture provides us but we can emulate it using the membarrier kernel
        // interface.
        //
        // This behaviour was documented in a patch, however it seems that it hasn't been
        // upstreamed yet Nevertheless it clearly explains the guarantees that the Linux kernel
        // provides us regarding the membarrier interface, and how to use it for JIT contexts.
        // https://lkml.kernel.org/lkml/07a8b963002cb955b7516e61bad19514a3acaa82.1623813516.git.luto@kernel.org/
        //
        // I couldn't find the follow up for that patch but there doesn't seem to be disagreement
        // about that specific part in the replies.
        // TODO: Check if the kernel has updated the membarrier documentation
        //
        // See the following issues for more info:
        //  * https://github.com/bytecodealliance/wasmtime/pull/3426
        //  * https://github.com/bytecodealliance/wasmtime/pull/4997
        //
        // TODO: x86 and s390x have coherent caches so they don't need this, but RISCV does not
        // guarantee that, so we may need to do something similar for it. However as noted in the
        // above kernel patch the SYNC_CORE membarrier has different guarantees on each
        // architecture so we need follow up and check what it provides us.
        // See: https://github.com/bytecodealliance/wasmtime/issues/5033
        #[cfg(feature = "std")]
        {
            match membarrier(MEMBARRIER_CMD_PRIVATE_EXPEDITED_SYNC_CORE) {
                Ok(_) => {}

                Err(e) if e.raw_os_error().map_or(false, |c| c == EPERM) => {
                    membarrier(MEMBARRIER_CMD_REGISTER_PRIVATE_EXPEDITED_SYNC_CORE)?;
                    membarrier(MEMBARRIER_CMD_PRIVATE_EXPEDITED_SYNC_CORE)?;
                }

                Err(e) if e.raw_os_error().map_or(false, |c| c == EINVAL) => {
                    membarrier(MEMBARRIER_CMD_GLOBAL)?;
                }

                Err(e) if e.raw_os_error().map_or(false, |c| c == ENOSYS) => {
                    if membarrier(MEMBARRIER_CMD_GLOBAL).is_err() {
                        // Both failed - membarrier unavailable (e.g. seccomp, restricted runner).
                        // Proceed without flush. Safe only when single-threaded.
                    }
                }

                e => e?,
            }
        }

        #[cfg(not(feature = "std"))]
        {
            match membarrier(MEMBARRIER_CMD_PRIVATE_EXPEDITED_SYNC_CORE) {
                Ok(_) => {}

                // EPERM happens if the calling process hasn't yet called the register membarrier.
                Err(e) if e == EPERM => {
                    membarrier(MEMBARRIER_CMD_REGISTER_PRIVATE_EXPEDITED_SYNC_CORE)?;
                    membarrier(MEMBARRIER_CMD_PRIVATE_EXPEDITED_SYNC_CORE)?;
                }

                // On kernels older than 4.16 the above syscall does not exist, so we can
                // fallback to MEMBARRIER_CMD_GLOBAL which has existed since 4.3.
                Err(e) if e == EINVAL => {
                    membarrier(MEMBARRIER_CMD_GLOBAL)?;
                }

                // ENOSYS: membarrier syscall not supported (e.g. seccomp, restricted runner).
                // Try GLOBAL; if both fail with ENOSYS, proceed without flush (single-threaded safe).
                Err(e) if e == ENOSYS => {
                    if let Err(e2) = membarrier(MEMBARRIER_CMD_GLOBAL) {
                        if e2 != ENOSYS {
                            return Err(e2);
                        }
                        // Both ENOSYS - membarrier unavailable. Proceed without flush.
                    }
                }

                e => e?,
            }
        }

        Ok(())
    }

    fn membarrier(barrier: libc::c_int) -> Result<()> {
        let flags: libc::c_int = 0;
        let res = unsafe { syscall(libc::SYS_membarrier, barrier, flags) };
        if res == 0 {
            Ok(())
        } else {
            #[cfg(feature = "std")]
            {
                Err(Error::last_os_error())
            }
            #[cfg(not(feature = "std"))]
            {
                let errno = unsafe { *libc::__errno_location() };
                Err(errno)
            }
        }
    }
}

#[cfg(not(all(
    target_arch = "aarch64",
    any(target_os = "linux", target_os = "android")
)))]
mod details {
    // NB: this uses `anyhow::Result` instead of `std::io::Result` to compile on
    // `no_std`.
    pub(crate) fn pipeline_flush_mt() -> super::Result<()> {
        Ok(())
    }
}

#[cfg(all(target_arch = "riscv64", target_os = "linux"))]
fn riscv_flush_icache(start: u64, end: u64) -> Result<()> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "one-core")] {
            use core::arch::asm;
            let _ = (start, end);
            unsafe {
                asm!("fence.i");
            };
            Ok(())
        } else {
            #[expect(non_upper_case_globals, reason = "matching C style")]
            match unsafe {
                libc::syscall(
                    {
                        // The syscall isn't defined in `libc`, so we define the syscall number here.
                        // https://github.com/torvalds/linux/search?q=__NR_arch_specific_syscall
                        const  __NR_arch_specific_syscall :i64 = 244;
                        // https://github.com/torvalds/linux/blob/5bfc75d92efd494db37f5c4c173d3639d4772966/tools/arch/riscv/include/uapi/asm/unistd.h#L40
                        const sys_riscv_flush_icache :i64 =  __NR_arch_specific_syscall + 15;
                        sys_riscv_flush_icache
                    },
                    // Currently these parameters are not used, but they are still defined.
                    start, // start
                    end, // end
                    {
                        const SYS_RISCV_FLUSH_ICACHE_LOCAL :i64 = 1;
                        const SYS_RISCV_FLUSH_ICACHE_ALL :i64 = SYS_RISCV_FLUSH_ICACHE_LOCAL;
                        SYS_RISCV_FLUSH_ICACHE_ALL
                    }, // flags
                )
            } {
                0 => { Ok(()) }
                _ => {
                    #[cfg(feature = "std")]
                    {
                        Err(std::io::Error::last_os_error())
                    }
                    #[cfg(not(feature = "std"))]
                    {
                        Err(-1)
                    }
                }
            }
        }
    }
}

pub(crate) use details::*;

/// See docs on [crate::clear_cache] for a description of what this function is trying to do.
#[inline]
pub(crate) fn clear_cache(_ptr: *const c_void, _len: usize) -> Result<()> {
    // TODO: On AArch64 we currently rely on the `mprotect` call that switches the memory from W+R
    // to R+X to do this for us, however that is an implementation detail and should not be relied
    // upon.
    // We should call some implementation of `clear_cache` here.
    //
    // See: https://github.com/bytecodealliance/wasmtime/issues/3310
    #[cfg(all(target_arch = "riscv64", target_os = "linux"))]
    riscv_flush_icache(_ptr as u64, (_ptr as u64) + (_len as u64))?;
    Ok(())
}
