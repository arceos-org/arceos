#![no_std]
#![no_main]

use axlog::ax_println;

extern crate alloc;
extern crate axstd;

mod coverage_runtime;

#[cfg(feature = "axtest_cov")]
fn dump_coverage_profraw() {
    use crate::alloc::string::ToString;
    ax_println!("[COVERAGE] Starting coverage dump...");
    unsafe extern "C" {
        fn __llvm_profile_get_size_for_buffer() -> u64;
        fn __llvm_profile_write_buffer(buffer: *mut u8) -> i32;
    }

    const COV_PATH: &str = "/axtest_cov.profraw";

    let size = unsafe { __llvm_profile_get_size_for_buffer() as usize };
    ax_println!("[COVERAGE] Size for buffer: {}", size);
    if size == 0 {
        ax_println!("coverage buffer is empty");
        return;
    }

    let mut buffer = alloc::vec![0u8; size];
    let ret = unsafe { __llvm_profile_write_buffer(buffer.as_mut_ptr()) };
    ax_println!("[COVERAGE] Write buffer returned: {}", ret);
    if ret != 0 {
        ax_println!("failed to write coverage buffer, code: {}", ret);
        return;
    }

    match axstd::fs::write(COV_PATH, &buffer) {
        Ok(()) => ax_println!("coverage_profraw dumped to {}", COV_PATH),
        Err(e) => ax_println!("failed to dump coverage_profraw to {}: {:?}", COV_PATH, e),
    }

    // ls /
    match axstd::fs::read_dir("/") {
        Ok(entries) => {
            ax_println!("Files in root directory:");
            for entry in entries {
                if let Ok(entry) = entry {
                    ax_println!(" - {}", entry.file_name().to_string());
                }
            }
        }
        Err(e) => ax_println!("failed to read root directory: {:?}", e),
    }
}

#[derive(Default)]
struct ThreadedExecutor;

impl axtest::AxTestExecutor for ThreadedExecutor {
    fn name(&self) -> &'static str {
        "thread"
    }

    fn run(
        &self,
        test_fn: fn() -> axtest::AxTestResult,
    ) -> Result<axtest::AxTestResult, &'static str> {
        let handle = axstd::thread::spawn(test_fn);
        handle.join().map_err(|_| "test thread join failed")
    }
}

#[unsafe(no_mangle)]
fn main() {
    ax_println!("Starting axtest runner...");
    let _summary = axtest::init()
        .add_executor(axtest::InlineExecutor)
        .add_executor(ThreadedExecutor)
        .set_default(ThreadedExecutor)
        .run_tests();

    #[cfg(feature = "axtest_cov")]
    dump_coverage_profraw();
}

// Define some example tests using the axtest framework
#[axtest::def_mod]
mod axtests {
    use axlog::ax_println;
    use axtest::{ax_assert_eq, ax_assert_ne, def_test};

    fn axtest_init(sym: axtest::AxTestDescriptor) {
        ax_println!("# hook init {}::{}", sym.module, sym.name);
    }

    fn axtest_exit(sym: axtest::AxTestDescriptor) {
        ax_println!("# hook exit {}::{}", sym.module, sym.name);
    }

    /// Simple addition test
    #[def_test(custom = "thread")]
    fn test_basic_addition() {
        let a = 2 + 2;
        ax_assert_eq!(a, 4);
    }

    /// String comparison test
    #[def_test(custom = "inner")]
    fn test_string_not_equal() {
        let s1 = "hello";
        let s2 = "world";
        ax_assert_ne!(s1, s2);
    }

    /// Ignored test example
    #[ignore = "demonstrate ignored case"]
    #[def_test]
    fn test_ignored_demo() {
        // This assertion should never run because the case is ignored.
        ax_assert_eq!(1, 2);
    }

    /// Failing test example (should panic)
    #[def_test]
    fn test_should_panic_demo() {
        // In axtest, an expected panic is modeled as an expected failed test body.
        ax_assert_eq!(1, 2);
    }
}
