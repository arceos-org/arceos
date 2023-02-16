pub struct ArchPerCpu;

impl ArchPerCpu {
    pub fn new() -> Self {
        Self
    }

    pub fn init(&mut self, _cpu_id: usize) {}
}
