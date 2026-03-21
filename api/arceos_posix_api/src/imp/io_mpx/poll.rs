use core::ffi::c_int;

use axerrno::{LinuxError, LinuxResult};
use axhal::time::wall_time;

use crate::{ctypes, imp::fd_ops::get_file_like};

/// Poll file descriptors for I/O readiness (POSIX `poll` semantics).
///
/// Returns the number of ready descriptors on success, 0 on timeout,
/// or a negative errno value on error.
pub fn sys_poll(fds: *mut ctypes::pollfd, nfds: ctypes::nfds_t, timeout: c_int) -> c_int {
    debug!(
        "sys_poll <= fds:{:#x} nfds:{} timeout:{}",
        fds as usize, nfds, timeout
    );
    syscall_body!(sys_poll, {
        if fds.is_null() && nfds > 0 {
            return Err(LinuxError::EFAULT);
        }

        let fds_slice = if nfds > 0 {
            unsafe { core::slice::from_raw_parts_mut(fds, nfds) }
        } else {
            &mut []
        };

        // Clear all revents
        for pfd in fds_slice.iter_mut() {
            pfd.revents = 0;
        }

        // Compute deadline
        let deadline = if timeout < 0 {
            None // block indefinitely
        } else if timeout == 0 {
            Some(wall_time()) // immediate, non-blocking
        } else {
            Some(wall_time() + core::time::Duration::from_millis(timeout as u64))
        };

        loop {
            #[cfg(feature = "net")]
            axnet::poll_interfaces();

            let mut ready_count: usize = 0;

            for pfd in fds_slice.iter_mut() {
                if pfd.fd < 0 {
                    // Negative fd: ignore (POSIX behavior)
                    continue;
                }

                match get_file_like(pfd.fd) {
                    Ok(file) => match file.poll() {
                        Ok(state) => {
                            let mut revents: i16 = 0;
                            if state.readable && (pfd.events & ctypes::POLLIN) != 0 {
                                revents |= ctypes::POLLIN;
                            }
                            if state.writable && (pfd.events & ctypes::POLLOUT) != 0 {
                                revents |= ctypes::POLLOUT;
                            }
                            if revents != 0 {
                                pfd.revents = revents;
                                ready_count += 1;
                            }
                        }
                        Err(_) => {
                            pfd.revents = ctypes::POLLERR;
                            ready_count += 1;
                        }
                    },
                    Err(_) => {
                        pfd.revents = ctypes::POLLNVAL;
                        ready_count += 1;
                    }
                }
            }

            if ready_count > 0 {
                return Ok(ready_count);
            }

            if deadline.is_some_and(|ddl| wall_time() >= ddl) {
                debug!("    poll timeout!");
                return Ok(0);
            }

            crate::sys_sched_yield();
        }
    })
}
