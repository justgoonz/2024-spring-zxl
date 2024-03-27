use core::ops::DerefMut;

//进程管理
use super::processor;
use super::*;
use crate::memory::{
    self,
    allocator::{ALLOCATOR, HEAP_SIZE},
    get_frame_alloc_for_sure, PAGE_SIZE,
};
use alloc::sync::Arc;
use alloc::{collections::*, format};
use spin::{Mutex, RwLock};
pub static PROCESS_MANAGER: spin::Once<ProcessManager> = spin::Once::new();

pub fn init(init: Arc<Process>) {
    // FIXME: set init process as Running
    let mut init_process = init.write().resume();

    // FIXME: set processor's current pid to init's pid
    // 处理器是单核的，因此同一时刻只有一个pid
    processor::set_pid(init.pid());
    //let process_ref = Arc::as_ref(&init);
    //let pid = process_ref.pid();
    //processor::set_pid(pid);

    PROCESS_MANAGER.call_once(|| ProcessManager::new(init));//进程管理器只初始化一次
}

pub fn get_process_manager() -> &'static ProcessManager {//获取进程管理器实例
    PROCESS_MANAGER
        .get()
        .expect("Process Manager has not been initialized")
}

pub struct ProcessManager {//进程管理器结构体
    //由RwLock和Mutex提供内部可变性
    processes: RwLock<BTreeMap<ProcessId, Arc<Process>>>, //储存所有进程
    ready_queue: Mutex<VecDeque<ProcessId>>,              //进程队列，存储pid
}

impl ProcessManager {
    pub fn new(init: Arc<Process>) -> Self {
        let mut processes = BTreeMap::new();
        let ready_queue = VecDeque::new();
        let pid = init.pid();

        trace!("Init {:#?}", init);

        processes.insert(pid, init);//添加初始化进程到进程集合中
        Self {//返回值
            processes: RwLock::new(processes),//用RwLock返回
            ready_queue: Mutex::new(ready_queue),//用Mutex返回
        }
    }

    #[inline]
    pub fn push_ready(&self, pid: ProcessId) {//将pid加入队列
        self.ready_queue.lock().push_back(pid);
    }

    #[inline]
    fn add_proc(&self, pid: ProcessId, proc: Arc<Process>) {//添加进程到进程集合中
        self.processes.write().insert(pid, proc);
    }

    #[inline]
    fn get_proc(&self, pid: &ProcessId) -> Option<Arc<Process>> {//获取指定pid的进程
        self.processes.read().get(pid).cloned()
    }

    pub fn current(&self) -> Arc<Process> {//获取当前进程
        self.get_proc(&processor::get_pid())
            .expect("No current process")
    }

    pub fn save_current(&self, context: &ProcessContext) {//保存当前处理器正在执行的进程，加入队列中
        let process_manager = get_process_manager();
        let current_pid = processor::get_pid();
        let current_process = process_manager.current();
        // FIXME: update current process's tick count
        current_process.write().tick();//记录进程的调度次数
        // FIXME: update current process's context
        //疑惑：为什么要传入一个参数修改上下文
        // FIXME: push current process to ready queue if still alive
        if current_process.read().status() != ProgramStatus::Dead {
            current_process.write().save(context);//保存当前进程上下文
        }
        process_manager.push_ready(current_pid);//将当前进程加入进程队列
    }

    pub fn switch_next(&self, context: &mut ProcessContext) -> ProcessId {//从队列中取出一个进程，加载到处理器中
        // FIXME: fetch the next process from ready queue
        // FIXME: check if the next process is ready,
        //        continue to fetch if not ready
        // FIXME: restore next process's context
        // FIXME: update processor's current pid
        let mut ready_queue = self.ready_queue.lock();//锁Mutex
        while let Some(next_pid) = ready_queue.pop_front(){//循环条件:队列非空
            if let Some(next_process) = self.get_proc(&next_pid){
                let mut next_process_inner = next_process.write();//锁RwLock
                if next_process_inner.status() == ProgramStatus::Ready{//进程状态为Ready
                    processor::set_pid(next_pid);  // 更新处理器的当前进程
                    //疑惑：为什么要传入一个参数修改上下文
                    next_process.write().restore(context);//恢复下一个进程的上下文
                    return next_pid; 
                }
            } 
        }
        KERNEL_PID
    }

    //创建一个新的内核线程
    pub fn spawn_kernel_thread(
        &self,
        entry: VirtAddr,
        name: String,
        proc_data: Option<ProcessData>,
    ) -> ProcessId {
        let kproc = self.get_proc(&KERNEL_PID).unwrap();
        let page_table = kproc.read().clone_page_table();
        let proc = Process::new(name, Some(Arc::downgrade(&kproc)), page_table, proc_data);

        // alloc stack for the new process base on pid
        let stack_top = proc.alloc_init_stack();

        // FIXME: set the stack frame

        // FIXME: add to process map

        // FIXME: push to ready queue

        KERNEL_PID
    }

    pub fn kill_current(&self, ret: isize) {//杀死当前进程
        self.kill(processor::get_pid(), ret);
    }
    //处理页面错误
    pub fn handle_page_fault(&self, addr: VirtAddr, err_code: PageFaultErrorCode) -> bool {
        // FIXME: handle page fault

        false
    }
    //杀死指定pid的进程
    pub fn kill(&self, pid: ProcessId, ret: isize) {
        let proc = self.get_proc(&pid);

        if proc.is_none() {
            warn!("Process #{} not found.", pid);
            return;
        }

        let proc = proc.unwrap();

        if proc.read().status() == ProgramStatus::Dead {
            warn!("Process #{} is already dead.", pid);
            return;
        }

        trace!("Kill {:#?}", &proc);

        proc.kill(ret);
    }
    //打印进程列表
    pub fn print_process_list(&self) {
        let mut output = String::from("  PID | PPID | Process Name |  Ticks  | Status\n");

        for (_, p) in self.processes.read().iter() {
            if p.read().status() != ProgramStatus::Dead {
                output += format!("{}\n", p).as_str();
            }
        }

        // TODO: print memory usage of kernel heap

        output += format!("Queue  : {:?}\n", self.ready_queue.lock()).as_str();

        output += &processor::print_processors();

        print!("{}", output);
    }
}
