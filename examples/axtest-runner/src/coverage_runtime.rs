// LLVM coverage runtime implementation for bare metal environments
// This provides the minimal runtime needed when compiled with -Zno-profiler-runtime

use core::{mem::size_of, ptr::addr_of};

const INSTR_PROF_RAW_MAGIC_64: u64 = ((255u64) << 56)
    | ((b'l' as u64) << 48)
    | ((b'p' as u64) << 40)
    | ((b'r' as u64) << 32)
    | ((b'o' as u64) << 24)
    | ((b'f' as u64) << 16)
    | ((b'r' as u64) << 8)
    | 129u64;
const INSTR_PROF_RAW_VERSION: u64 = 10;
const VALUE_KIND_LAST: u64 = 2;
const VARIANT_MASK_BYTE_COVERAGE: u64 = 1u64 << 60;

#[derive(Copy, Clone)]
struct Section {
    start: usize,
    end: usize,
}

impl Section {
    #[inline(always)]
    fn len(self) -> usize {
        self.end.saturating_sub(self.start)
    }
}

struct Layout {
    data: Section,
    counters: Section,
    names: Section,
    vnds: Section,
    pad_before_counters: usize,
    pad_after_counters: usize,
    pad_after_bitmap: usize,
    pad_after_names: usize,
    version: u64,
    counter_entry_size: usize,
}

impl Layout {
    fn collect() -> Self {
        let data = Section {
            start: addr_of!(__start___llvm_prf_data) as usize,
            end: addr_of!(__stop___llvm_prf_data) as usize,
        };
        let counters = Section {
            start: addr_of!(__start___llvm_prf_cnts) as usize,
            end: addr_of!(__stop___llvm_prf_cnts) as usize,
        };
        let names = Section {
            start: addr_of!(__start___llvm_prf_names) as usize,
            end: addr_of!(__stop___llvm_prf_names) as usize,
        };
        let vnds = Section {
            start: addr_of!(__start___llvm_prf_vnds) as usize,
            end: addr_of!(__stop___llvm_prf_vnds) as usize,
        };

        let version = INSTR_PROF_RAW_VERSION;
        let counter_entry_size = counter_entry_size(version);
        let pad_before_counters = align_up(data.len(), 8) - data.len();
        let pad_after_counters = 0usize;
        let pad_after_bitmap = 0usize;
        let pad_after_names = align_up(names.len(), 8) - names.len();

        Self {
            data,
            counters,
            names,
            vnds,
            pad_before_counters,
            pad_after_counters,
            pad_after_bitmap,
            pad_after_names,
            version,
            counter_entry_size,
        }
    }

    fn total_size(&self) -> usize {
        size_of::<RawHeader>()
            + self.data.len()
            + self.pad_before_counters
            + self.counters.len()
            + self.pad_after_counters
            + self.pad_after_bitmap
            + self.names.len()
            + self.pad_after_names
    }

    fn header(&self) -> RawHeader {
        RawHeader {
            magic: INSTR_PROF_RAW_MAGIC_64,
            version: self.version,
            binary_ids_size: 0,
            num_data: (self.data.len() / 64) as u64,
            padding_bytes_before_counters: self.pad_before_counters as u64,
            num_counters: self.counters.len().div_ceil(self.counter_entry_size) as u64,
            padding_bytes_after_counters: self.pad_after_counters as u64,
            num_bitmap_bytes: 0,
            padding_bytes_after_bitmap_bytes: self.pad_after_bitmap as u64,
            names_size: self.names.len() as u64,
            counters_delta: self.counters.start.wrapping_sub(self.data.start) as u64,
            bitmap_delta: 0,
            names_delta: self.names.start as u64,
            num_vtables: 0,
            vnames_size: 0,
            value_kind_last: VALUE_KIND_LAST,
        }
    }
}

#[repr(C)]
struct RawHeader {
    magic: u64,
    version: u64,
    binary_ids_size: u64,
    num_data: u64,
    padding_bytes_before_counters: u64,
    num_counters: u64,
    padding_bytes_after_counters: u64,
    num_bitmap_bytes: u64,
    padding_bytes_after_bitmap_bytes: u64,
    names_size: u64,
    counters_delta: u64,
    bitmap_delta: u64,
    names_delta: u64,
    num_vtables: u64,
    vnames_size: u64,
    value_kind_last: u64,
}

// Link with the profiling sections defined in linker script
unsafe extern "C" {
    // Data section
    static __start___llvm_prf_data: u8;
    static __stop___llvm_prf_data: u8;

    // Counters section
    static __start___llvm_prf_cnts: u8;
    static __stop___llvm_prf_cnts: u8;

    // Names section
    static __start___llvm_prf_names: u8;
    static __stop___llvm_prf_names: u8;

    // Value profiling data
    static __start___llvm_prf_vnds: u8;
    static __stop___llvm_prf_vnds: u8;
}

#[inline(always)]
fn align_up(value: usize, align: usize) -> usize {
    (value + align - 1) & !(align - 1)
}

#[inline(always)]
fn write_u64_le(out: &mut [u8], offset: &mut usize, value: u64) -> Result<(), i32> {
    if *offset + 8 > out.len() {
        return Err(1);
    }
    out[*offset..*offset + 8].copy_from_slice(&value.to_le_bytes());
    *offset += 8;
    Ok(())
}

#[inline(always)]
fn write_zeros(out: &mut [u8], offset: &mut usize, len: usize) -> Result<(), i32> {
    if *offset + len > out.len() {
        return Err(1);
    }
    out[*offset..*offset + len].fill(0);
    *offset += len;
    Ok(())
}

#[inline(always)]
fn copy_section(out: &mut [u8], offset: &mut usize, section: Section) -> Result<(), i32> {
    let len = section.len();
    if *offset + len > out.len() {
        return Err(1);
    }
    unsafe {
        core::ptr::copy_nonoverlapping(section.start as *const u8, out.as_mut_ptr().add(*offset), len)
    };
    *offset += len;
    Ok(())
}

#[inline(always)]
fn counter_entry_size(version: u64) -> usize {
    if (version & VARIANT_MASK_BYTE_COVERAGE) != 0 {
        1
    } else {
        size_of::<u64>()
    }
}

/// Calculate the size of coverage data to be written
pub fn llvm_profile_get_size_for_buffer() -> usize {
    Layout::collect().total_size()
}

/// Write coverage data to a buffer
pub fn llvm_profile_write_buffer(buffer: *mut u8) -> i32 {
    if buffer.is_null() {
        return 1; // Error
    }

    let layout = Layout::collect();
    let expected_size = layout.total_size();
    let out = unsafe { core::slice::from_raw_parts_mut(buffer, expected_size) };

    let header = layout.header();

    let mut offset = 0usize;

    for value in [
        header.magic,
        header.version,
        header.binary_ids_size,
        header.num_data,
        header.padding_bytes_before_counters,
        header.num_counters,
        header.padding_bytes_after_counters,
        header.num_bitmap_bytes,
        header.padding_bytes_after_bitmap_bytes,
        header.names_size,
        header.counters_delta,
        header.bitmap_delta,
        header.names_delta,
        header.num_vtables,
        header.vnames_size,
        header.value_kind_last,
    ] {
        if write_u64_le(out, &mut offset, value).is_err() {
            return 1;
        }
    }

    if copy_section(out, &mut offset, layout.data).is_err() {
        return 1;
    }
    if write_zeros(out, &mut offset, layout.pad_before_counters).is_err() {
        return 1;
    }
    if copy_section(out, &mut offset, layout.counters).is_err() {
        return 1;
    }
    if copy_section(out, &mut offset, layout.names).is_err() {
        return 1;
    }
    if write_zeros(out, &mut offset, layout.pad_after_names).is_err() {
        return 1;
    }

    // Value profiling nodes are emitted in memory sections, but the raw format
    // stores serialized value records. Keep it empty when there is no value data.
    if layout.vnds.len() != 0 {
        // vnds has data, but we do not serialize it directly into raw payload.
    }

    if offset != expected_size {
        return 1;
    }

    0
}
