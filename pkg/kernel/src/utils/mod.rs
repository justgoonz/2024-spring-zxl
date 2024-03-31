#[macro_use]
mod macros;
#[macro_use]
mod regs;

//pub mod clock;
pub mod func;
pub mod logger;

pub use macros::*;
pub use regs::*;

use crate::proc::*;
use alloc:: format;
pub const fn get_ascii_header() -> &'static str {
    concat!(
        r"
__  __      __  _____            ____  _____
\ \/ /___ _/ /_/ ___/___  ____  / __ \/ ___/
 \  / __ `/ __/\__ \/ _ \/ __ \/ / / /\__ \
 / / /_/ / /_ ___/ /  __/ / / / /_/ /___/ /
/_/\__,_/\__//____/\___/_/ /_/\____//____/

                                       v",
        env!("CARGO_PKG_VERSION")
    )
}

pub fn new_test_thread(id: &str) -> ProcessId {
    let mut proc_data = ProcessData::new();
    proc_data.set_env("id", id);

    spawn_kernel_thread(
        func::test,
        format!("#{}_test", id),//format!调用alloc::format?
        Some(proc_data),
    )
}

pub fn new_stack_test_thread() {//测试"创建内核线程"
    let pid = spawn_kernel_thread(
        func::stack_test,
        alloc::string::String::from("stack"),
        None,
    );

    // wait for progress exit
    wait(pid);
}

//为 ProcessManager 添加相关的处理函数，使得外部函数可以获取指定进程的返回值。
fn wait(pid: ProcessId) {
    let proc_manager = manager::get_process_manager();//获取process_manager
    loop {
        // FIXME: try to get the status of the process
        
        // HINT: it's better to use the exit code
        if let Some(exit_code) =  proc_manager.get_exit_code(pid){//进程已退出
            x86_64::instructions::hlt();//使cpu进入休眠状态
        } else {//进程处于running状态
            break;//出循环
        }
    }
}


const SHORT_UNITS: [&str; 4] = ["B", "K", "M", "G"];
const UNITS: [&str; 4] = ["B", "KiB", "MiB", "GiB"];

pub fn humanized_size(size: u64) -> (f32, &'static str) {
    humanized_size_impl(size, false)
}

pub fn humanized_size_short(size: u64) -> (f32, &'static str) {
    humanized_size_impl(size, true)
}

#[inline]
pub fn humanized_size_impl(size: u64, short: bool) -> (f32, &'static str) {
    let bytes = size as f32;

    let units = if short { &SHORT_UNITS } else { &UNITS };

    let mut unit = 0;
    let mut bytes = bytes;

    while bytes >= 1024f32 && unit < units.len() {
        bytes /= 1024f32;
        unit += 1;
    }

    (bytes, units[unit])
}