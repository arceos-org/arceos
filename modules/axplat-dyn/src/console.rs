use axplat::console::ConsoleIf;

struct ConsoleIfImpl;

#[impl_plat_interface]
impl ConsoleIf for ConsoleIfImpl {
    /// Writes given bytes to the console.
    fn write_bytes(bytes: &[u8]) {
        let s = core::str::from_utf8(bytes).unwrap_or_default();
        let mut remaining = s;
        while let Some(pos) = remaining.find('\n') {
            // 打印 '\n' 之前的部分
            somehal::console::_write_str(&remaining[..pos]);
            // 打印 "\r\n"
            somehal::console::_write_str("\r\n");
            // 继续处理剩余部分
            remaining = &remaining[pos + 1..];
        }
        // 打印最后剩余的部分（如果有的话）
        if !remaining.is_empty() {
            somehal::console::_write_str(remaining);
        }
    }

    /// Reads bytes from the console into the given mutable slice.
    ///
    /// Returns the number of bytes read.
    fn read_bytes(bytes: &mut [u8]) -> usize {
        todo!()
        // let mut read_len = 0;
        // while read_len < bytes.len() {
        //     if let Some(c) = somehal::console::getchar() {
        //         bytes[read_len] = c;
        //     } else {
        //         break;
        //     }
        //     read_len += 1;
        // }
        // read_len
    }

    /// Returns the IRQ number for the console input interrupt.
    ///
    /// Returns `None` if input interrupt is not supported.
    #[cfg(feature = "irq")]
    fn irq_num() -> Option<usize> {
        None
    }
}
