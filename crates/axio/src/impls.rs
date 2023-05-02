use crate::{prelude::*, Result};
use core::cmp;

impl Read for &[u8] {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let amt = cmp::min(buf.len(), self.len());
        let a = &self[..amt];
        let b = &self[amt..];

        // First check if the amount of bytes we want to read is small:
        // `copy_from_slice` will generally expand to a call to `memcpy`, and
        // for a single byte the overhead is significant.
        if amt == 1 {
            buf[0] = a[0];
        } else {
            buf[..amt].copy_from_slice(a);
        }

        *self = b;
        Ok(amt)
    }

    #[inline]
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        if buf.len() > self.len() {
            return axerrno::ax_err!(UnexpectedEof, "failed to fill whole buffer");
        }
        let amt = buf.len();
        let a = &self[..amt];
        let b = &self[amt..];

        // First check if the amount of bytes we want to read is small:
        // `copy_from_slice` will generally expand to a call to `memcpy`, and
        // for a single byte the overhead is significant.
        if amt == 1 {
            buf[0] = a[0];
        } else {
            buf[..amt].copy_from_slice(a);
        }

        *self = b;
        Ok(())
    }

    #[inline]
    #[cfg(feature = "alloc")]
    fn read_to_end(&mut self, buf: &mut alloc::vec::Vec<u8>) -> Result<usize> {
        buf.extend_from_slice(self);
        let len = self.len();
        *self = &self[len..];
        Ok(len)
    }
}
