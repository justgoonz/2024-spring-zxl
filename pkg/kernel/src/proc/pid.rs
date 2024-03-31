use core::sync::atomic::{AtomicU16, Ordering};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProcessId(pub u16);
static NEXT_PID: AtomicU16 = AtomicU16::new(1); // 开始于 1 以避免将 0 作为有效的 PID
impl ProcessId {
    pub fn new() -> Self {
        // FIXME: Get a unique PID
        let pid = NEXT_PID.fetch_add(1, Ordering::SeqCst);//这部分的实现是否正确？
        ProcessId(pid)
    }
}

impl Default for ProcessId {
    fn default() -> Self {
        Self::new()
    }
}

impl core::fmt::Display for ProcessId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl core::fmt::Debug for ProcessId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<ProcessId> for u16 {
    fn from(pid: ProcessId) -> Self {
        pid.0
    }
}
