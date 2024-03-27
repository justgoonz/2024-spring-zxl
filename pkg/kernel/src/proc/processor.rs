//cpu核心结构体
use core::sync::atomic::{AtomicU16, Ordering};

use crate::proc::ProcessId;
use alloc::{string::String, vec::Vec};
use x86::cpuid::CpuId;

const MAX_CPU_COUNT: usize = 4;

#[allow(clippy::declare_interior_mutable_const)]
const EMPTY: Processor = Processor::new(); // means no process

static PROCESSORS: [Processor; MAX_CPU_COUNT] = [EMPTY; MAX_CPU_COUNT];

/// Returns the current processor based on the current APIC ID
/// 返回当前处理器的引用
fn current() -> &'static Processor {
    let cpuid = CpuId::new()
        .get_feature_info()
        .unwrap()
        .initial_local_apic_id() as usize;

    &PROCESSORS[cpuid]
}

pub fn print_processors() -> String {//打印所有处理器及进程id
    alloc::format!(
        "CPUs   : {}\n",
        PROCESSORS
            .iter()
            .enumerate()
            .filter(|(_, p)| !p.is_free())
            .map(|(i, p)| alloc::format!("[{}: {}]", i, p.get_pid().unwrap()))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

/// Processor holds the current process id
pub struct Processor(AtomicU16);

impl Processor {
    pub const fn new() -> Self {
        Self(AtomicU16::new(0))
    }
}

#[inline]
pub fn set_pid(pid: ProcessId) {//设置当前处理器运行的pid
    current().set_pid(pid)
}

#[inline]
pub fn get_pid() -> ProcessId {//获取当前处理器运行的pid
    current().get_pid().expect("No current process")
}

impl Processor {
    #[inline]
    pub fn is_free(&self) -> bool {//检查处理器是否空闲
        self.0.load(Ordering::Relaxed) == 0
    }

    #[inline]
    pub fn set_pid(&self, pid: ProcessId) {//设置处理器上运行的进程id
        self.0.store(pid.0, Ordering::Relaxed);
    }

    #[inline]
    pub fn get_pid(&self) -> Option<ProcessId> {//获取正在运行的pid
        let pid = self.0.load(Ordering::Relaxed);
        if pid == 0 {
            None
        } else {
            Some(ProcessId(pid))
        }
    }
}
