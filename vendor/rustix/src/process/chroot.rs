use crate::{backend, io, path};

/// `chroot(path)`—Change the process root directory.
///
/// # References
///  - [Linux]
///
/// [Linux]: https://man7.org/linux/man-pages/man2/chroot.2.html
#[inline]
pub fn chroot<P: path::Arg>(path: P) -> io::Result<()> {
    path.into_with_c_str(backend::process::syscalls::chroot)
}
