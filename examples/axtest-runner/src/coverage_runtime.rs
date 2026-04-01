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
fn section_size(start: usize, end: usize) -> usize {
    end.saturating_sub(start)
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
#[unsafe(no_mangle)]
pub extern "C" fn __llvm_profile_get_size_for_buffer() -> u64 {
    let data_start = addr_of!(__start___llvm_prf_data) as usize;
    let data_end = addr_of!(__stop___llvm_prf_data) as usize;
    let cnts_start = addr_of!(__start___llvm_prf_cnts) as usize;
    let cnts_end = addr_of!(__stop___llvm_prf_cnts) as usize;
    let names_start = addr_of!(__start___llvm_prf_names) as usize;
    let names_end = addr_of!(__stop___llvm_prf_names) as usize;

    let data_size = section_size(data_start, data_end);
    let cnts_size = section_size(cnts_start, cnts_end);
    let names_size = section_size(names_start, names_end);

    let pad_before_counters = align_up(data_size, 8) - data_size;
    let pad_after_counters = 0usize;
    let pad_after_bitmap = 0usize;
    let pad_after_names = align_up(names_size, 8) - names_size;

    let total = size_of::<RawHeader>()
        + data_size
        + pad_before_counters
        + cnts_size
        + pad_after_counters
        + pad_after_bitmap
        + names_size;
    let total = total + pad_after_names;

    total as u64
}

/// Write coverage data to a buffer
#[unsafe(no_mangle)]
pub extern "C" fn __llvm_profile_write_buffer(buffer: *mut u8) -> i32 {
    if buffer.is_null() {
        return 1; // Error
    }

    let data_start = addr_of!(__start___llvm_prf_data) as usize;
    let data_end = addr_of!(__stop___llvm_prf_data) as usize;
    let cnts_start = addr_of!(__start___llvm_prf_cnts) as usize;
    let cnts_end = addr_of!(__stop___llvm_prf_cnts) as usize;
    let names_start = addr_of!(__start___llvm_prf_names) as usize;
    let names_end = addr_of!(__stop___llvm_prf_names) as usize;
    let vnds_start = addr_of!(__start___llvm_prf_vnds) as usize;
    let vnds_end = addr_of!(__stop___llvm_prf_vnds) as usize;

    let data_size = section_size(data_start, data_end);
    let cnts_size = section_size(cnts_start, cnts_end);
    let names_size = section_size(names_start, names_end);
    let vnds_size = section_size(vnds_start, vnds_end);
    let version = INSTR_PROF_RAW_VERSION;
    let ctr_size = counter_entry_size(version);

    let pad_before_counters = align_up(data_size, 8) - data_size;
    let pad_after_counters = 0usize;
    let pad_after_bitmap = 0usize;
    let pad_after_names = align_up(names_size, 8) - names_size;

    let expected_size = __llvm_profile_get_size_for_buffer() as usize;
    let out = unsafe { core::slice::from_raw_parts_mut(buffer, expected_size) };

    let header = RawHeader {
        magic: INSTR_PROF_RAW_MAGIC_64,
        version,
        binary_ids_size: 0,
        num_data: (data_size / 64) as u64,
        padding_bytes_before_counters: pad_before_counters as u64,
        num_counters: (cnts_size.div_ceil(ctr_size)) as u64,
        padding_bytes_after_counters: pad_after_counters as u64,
        num_bitmap_bytes: 0,
        padding_bytes_after_bitmap_bytes: pad_after_bitmap as u64,
        names_size: names_size as u64,
        counters_delta: cnts_start.wrapping_sub(data_start) as u64,
        bitmap_delta: 0,
        names_delta: names_start as u64,
        num_vtables: 0,
        vnames_size: 0,
        value_kind_last: VALUE_KIND_LAST,
    };

    let mut offset = 0usize;

    macro_rules! write_u64 {
        ($val:expr) => {{
            if offset + 8 > out.len() {
                return 1;
            }
            let bytes = ($val).to_le_bytes();
            out[offset..offset + 8].copy_from_slice(&bytes);
            offset += 8;
        }};
    }

    write_u64!(header.magic);
    write_u64!(header.version);
    write_u64!(header.binary_ids_size);
    write_u64!(header.num_data);
    write_u64!(header.padding_bytes_before_counters);
    write_u64!(header.num_counters);
    write_u64!(header.padding_bytes_after_counters);
    write_u64!(header.num_bitmap_bytes);
    write_u64!(header.padding_bytes_after_bitmap_bytes);
    write_u64!(header.names_size);
    write_u64!(header.counters_delta);
    write_u64!(header.bitmap_delta);
    write_u64!(header.names_delta);
    write_u64!(header.num_vtables);
    write_u64!(header.vnames_size);
    write_u64!(header.value_kind_last);

    if offset + data_size > out.len() {
        return 1;
    }
    unsafe {
        core::ptr::copy_nonoverlapping(
            data_start as *const u8,
            out.as_mut_ptr().add(offset),
            data_size,
        )
    };
    offset += data_size;

    if offset + pad_before_counters > out.len() {
        return 1;
    }
    out[offset..offset + pad_before_counters].fill(0);
    offset += pad_before_counters;

    if offset + cnts_size > out.len() {
        return 1;
    }
    unsafe {
        core::ptr::copy_nonoverlapping(
            cnts_start as *const u8,
            out.as_mut_ptr().add(offset),
            cnts_size,
        )
    };
    offset += cnts_size;

    if offset + names_size > out.len() {
        return 1;
    }
    unsafe {
        core::ptr::copy_nonoverlapping(
            names_start as *const u8,
            out.as_mut_ptr().add(offset),
            names_size,
        )
    };
    offset += names_size;

    if offset + pad_after_names > out.len() {
        return 1;
    }
    out[offset..offset + pad_after_names].fill(0);
    offset += pad_after_names;

    // Value profiling nodes are emitted in memory sections, but the raw format
    // stores serialized value records. Keep it empty when there is no value data.
    if vnds_size != 0 {
        // vnds has data, but we do not serialize it directly into raw payload.
    }

    if offset != expected_size {
        return 1;
    }

    0
}
