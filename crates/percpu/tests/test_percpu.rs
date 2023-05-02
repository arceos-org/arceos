#![cfg(not(target_os = "macos"))]

use percpu::*;

// Initial value is unsupported for testing.

#[def_percpu]
static BOOL: bool = false;

#[def_percpu]
static U8: u8 = 0;

#[def_percpu]
static U16: u16 = 0;

#[def_percpu]
static U32: u32 = 0;

#[def_percpu]
static U64: u64 = 0;

#[def_percpu]
static USIZE: usize = 0;

struct Struct {
    foo: usize,
    bar: u8,
}

#[def_percpu]
static STRUCT: Struct = Struct { foo: 0, bar: 0 };

#[cfg(target_os = "linux")]
#[test]
fn test_percpu() {
    println!("feature = \"sp-naive\": {}", cfg!(feature = "sp-naive"));

    #[cfg(feature = "sp-naive")]
    let base = 0;

    #[cfg(not(feature = "sp-naive"))]
    let base = {
        init(4);
        set_local_thread_pointer(0);

        let base = get_local_thread_pointer();
        println!("per-CPU area base = {:#x}", base);
        println!("per-CPU area size = {}", percpu_area_size());
        base
    };

    println!("bool offset: {:#x}", BOOL.offset());
    println!("u8 offset: {:#x}", U8.offset());
    println!("u16 offset: {:#x}", U16.offset());
    println!("u32 offset: {:#x}", U32.offset());
    println!("u64 offset: {:#x}", U64.offset());
    println!("usize offset: {:#x}", USIZE.offset());
    println!("struct offset: {:#x}", STRUCT.offset());
    println!();

    unsafe {
        assert_eq!(base + BOOL.offset(), BOOL.current_ptr() as usize);
        assert_eq!(base + U8.offset(), U8.current_ptr() as usize);
        assert_eq!(base + U16.offset(), U16.current_ptr() as usize);
        assert_eq!(base + U32.offset(), U32.current_ptr() as usize);
        assert_eq!(base + U64.offset(), U64.current_ptr() as usize);
        assert_eq!(base + USIZE.offset(), USIZE.current_ptr() as usize);
        assert_eq!(base + STRUCT.offset(), STRUCT.current_ptr() as usize);
    }

    BOOL.write_current(true);
    U8.write_current(123);
    U16.write_current(0xabcd);
    U32.write_current(0xdead_beef);
    U64.write_current(0xa2ce_a2ce_a2ce_a2ce);
    USIZE.write_current(0xffff_0000);

    STRUCT.with_current(|s| {
        s.foo = 0x2333;
        s.bar = 100;
    });

    println!("bool value: {}", BOOL.read_current());
    println!("u8 value: {}", U8.read_current());
    println!("u16 value: {:#x}", U16.read_current());
    println!("u32 value: {:#x}", U32.read_current());
    println!("u64 value: {:#x}", U64.read_current());
    println!("usize value: {:#x}", USIZE.read_current());

    assert_eq!(U8.read_current(), 123);
    assert_eq!(U16.read_current(), 0xabcd);
    assert_eq!(U32.read_current(), 0xdead_beef);
    assert_eq!(U64.read_current(), 0xa2ce_a2ce_a2ce_a2ce);
    assert_eq!(USIZE.read_current(), 0xffff_0000);

    STRUCT.with_current(|s| {
        println!("struct.foo value: {:#x}", s.foo);
        println!("struct.bar value: {}", s.bar);
        assert_eq!(s.foo, 0x2333);
        assert_eq!(s.bar, 100);
    });
}
