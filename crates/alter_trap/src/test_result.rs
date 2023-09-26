#[derive(Clone, Copy)]
#[repr(C)]
pub struct TestResult {
    value: usize,
    cause: usize,
}

impl TestResult {
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
