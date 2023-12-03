//! TestResult

/// 传递检测结果。
/// value 为返回值，cause 为可能的出错原因（在 riscv64 下是触发异常时的 scause）。
/// 当 cause = 0 时，表示没有发生异常。
/// 
/// 可以被转换成 `Result` / `Option` / `bool` 使用
/// 
/// ```
/// let passed = TestResult { value: 5, cause: 0 }
/// let failed = TestResult { value: 10, cause: TestResult::LOAD_PAGE_FAULT }
/// }
/// assert!(passed.as_result(), Ok(5));
/// assert!(failed.as_result(), Err(TestResult::LOAD_PAGE_FAULT));
/// assert!(passed.as_option()), Some(5));
/// assert!(failed.as_option(), None);
/// assert!(passed.as_bool()), true);
/// assert!(failed.as_bool(), false);
/// ```
#[derive(Clone, Copy)]
#[repr(C)]
pub struct TestResult {
    pub value: usize,
    pub cause: usize,
}

impl TestResult {
    pub const OK: usize = 0;
    pub const LOAD_PAGE_FAULT: usize = 13;
    pub const STORE_PAGE_FAULT:usize = 15;

    pub fn as_result(self) -> Result<usize, usize> {
        self.into()
    }
    pub fn as_option(self) -> Option<usize> {
        self.into()
    }
    pub fn as_bool(self) -> bool {
        self.into()
    }
}

impl Into<Result<usize, usize>> for TestResult {
    fn into(self) -> Result<usize, usize> {
        if self.cause == 0 {
            Ok(self.value)
        } else {
            Err(self.cause)
        }
    }
}

impl Into<Option<usize>> for TestResult {
    fn into(self) -> Option<usize> {
        if self.cause == 0 {
            Some(self.value)
        } else {
            None
        }
    }
}

impl Into<bool> for TestResult {
    fn into(self) -> bool {
        self.cause == 0
    }
}
