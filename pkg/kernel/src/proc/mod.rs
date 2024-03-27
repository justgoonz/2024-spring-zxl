mod context;
mod data;
mod manager;
mod paging;
mod pid;
mod process;
mod processor;

use core::ops::DerefMut;

use manager::*;
use process::*;
use crate::memory::PAGE_SIZE;

use alloc::string::String;
pub use context::ProcessContext;
pub use paging::PageTableContext;
pub use data::ProcessData;
pub use pid::ProcessId;

use x86_64::structures::idt::PageFaultErrorCode;
use x86_64::VirtAddr;
use crate::alloc::string::ToString;

// 0xffff_ff00_0000_0000 is the kernel's address space
pub const STACK_MAX: u64 = 0x0000_4000_0000_0000;

pub const STACK_MAX_PAGES: u64 = 0x100000;
pub const STACK_MAX_SIZE: u64 = STACK_MAX_PAGES * PAGE_SIZE;
pub const STACK_START_MASK: u64 = !(STACK_MAX_SIZE - 1);
// [bot..0x2000_0000_0000..top..0x3fff_ffff_ffff]
// init stack(栈)
pub const STACK_DEF_PAGE: u64 = 1;
pub const STACK_DEF_SIZE: u64 = STACK_DEF_PAGE * PAGE_SIZE;
pub const STACK_INIT_BOT: u64 = STACK_MAX - STACK_DEF_SIZE;
pub const STACK_INIT_TOP: u64 = STACK_MAX - 8;
// [bot..0xffffff0100000000..top..0xffffff01ffffffff]
// kernel stack(内核栈)
pub const KSTACK_MAX: u64 = 0xffff_ff02_0000_0000;
//设置内核栈的默认页数
pub const KSTACK_DEF_PAGE: u64 = 8;/* FIXME: decide on the boot config */
pub const KSTACK_DEF_SIZE: u64 = KSTACK_DEF_PAGE * PAGE_SIZE;
pub const KSTACK_INIT_BOT: u64 = KSTACK_MAX - KSTACK_DEF_SIZE;//栈底
pub const KSTACK_INIT_TOP: u64 = KSTACK_MAX - 8;

pub const KERNEL_PID: ProcessId = ProcessId(1);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ProgramStatus {
    Running,
    Ready,
    Blocked,
    Dead,
}

/// init process manager
pub fn init() {
    let mut kproc_data = ProcessData::new();//内核栈

    // FIXME: set the kernel stack
    kproc_data.set_stack(VirtAddr::new(KSTACK_INIT_BOT),KSTACK_DEF_SIZE);
    // 是否需要kproc_data.set_env()
    //kproc_data.set_env();
    trace!("Init process data: {:#?}", kproc_data);

    // kernel process
    //let kproc = { /* FIXME: create kernel process */ };
    let kproc = Process::new(
        //"kernel".to_string(), 这个依赖于std，需要使用&static str实现
        "kernel".to_string(),
        None,
        PageTableContext::new(),
        Some(kproc_data)
    );
    
    manager::init(kproc);

    info!("Process Manager Initialized.");
}
//切换到下一个进程
pub fn switch(context: &mut ProcessContext) {//参数是当前处理器的上下文
    x86_64::instructions::interrupts::without_interrupts(|| {//确保在关闭中断的状态下继续执行
        let process_manager = get_process_manager();
        process_manager.save_current(context);//保存当前上下文
        process_manager.switch_next(context);//加载新的上下文
    });
}
//创建一个新的内核线程
pub fn spawn_kernel_thread(entry: fn() -> !, name: String, data: Option<ProcessData>) -> ProcessId {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let entry = VirtAddr::new(entry as usize as u64);
        get_process_manager().spawn_kernel_thread(entry, name, data)
    })
}
//打印进程列表
pub fn print_process_list() {
    x86_64::instructions::interrupts::without_interrupts(|| {
        get_process_manager().print_process_list();
    })
}
//获取当前进程的环境变量
pub fn env(key: &str) -> Option<String> {
    x86_64::instructions::interrupts::without_interrupts(|| {
        // FIXME: get current process's environment variable
        //获取当前进程的读锁
        get_process_manager().current().read().env(key)
    })
}
//退出当前进程
pub fn process_exit(ret: isize) -> ! {
    x86_64::instructions::interrupts::without_interrupts(|| {
        get_process_manager().kill_current(ret);
    });

    loop {
        x86_64::instructions::hlt();
    }
}
//处理页面错误
pub fn handle_page_fault(addr: VirtAddr, err_code: PageFaultErrorCode) -> bool {
    x86_64::instructions::interrupts::without_interrupts(|| {
        get_process_manager().handle_page_fault(addr, err_code)
    })
}
