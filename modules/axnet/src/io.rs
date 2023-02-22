use alloc::{string::String, vec::Vec};

use axerror::{ax_err, AxResult};

pub trait Read {
    fn read(&mut self, buf: &mut [u8]) -> AxResult<usize>;

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> AxResult<usize> {
        let start_len = buf.len();
        let mut probe = [0u8; 32];
        loop {
            match self.read(&mut probe) {
                Ok(0) => return Ok(buf.len() - start_len),
                Ok(n) => buf.extend_from_slice(&probe[..n]),
                Err(e) => return Err(e),
            }
        }
    }

    fn read_to_string(&mut self, buf: &mut String) -> AxResult<usize> {
        let old_len = buf.len();
        let buf = unsafe { buf.as_mut_vec() };
        let ret = self.read_to_end(buf)?;
        if core::str::from_utf8(&buf[old_len..]).is_err() {
            ax_err!(Io, "stream did not contain valid UTF-8")
        } else {
            Ok(ret)
        }
    }

    fn read_exact(&mut self, mut buf: &mut [u8]) -> AxResult {
        while !buf.is_empty() {
            match self.read(buf) {
                Ok(0) => break,
                Ok(n) => {
                    let tmp = buf;
                    buf = &mut tmp[n..];
                }
                Err(e) => return Err(e),
            }
        }
        if !buf.is_empty() {
            ax_err!(Io, "failed to fill whole buffer")
        } else {
            Ok(())
        }
    }
}

pub trait Write {
    fn write(&mut self, buf: &[u8]) -> AxResult<usize>;
    fn flush(&mut self) -> AxResult;

    fn write_all(&mut self, mut buf: &[u8]) -> AxResult {
        while !buf.is_empty() {
            match self.write(buf) {
                Ok(0) => return ax_err!(Io, "failed to write whole buffer"),
                Ok(n) => buf = &buf[n..],
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }
}
