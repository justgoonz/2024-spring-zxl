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
        entry: VirtAddr,//待执行的函数入口地址
        name: String,
        proc_data: Option<ProcessData>,
    ) -> ProcessId {
        let kproc = self.get_proc(&KERNEL_PID).unwrap();
        let page_table = kproc.read().clone_page_table();
        let proc = Process::new(name, Some(Arc::downgrade(&kproc)), page_table, proc_data);
        let pid = proc.pid();
        // alloc stack for the new process base on pid
        let stack_top = proc.alloc_init_stack();//分配初始栈，返回栈顶地址
        // FIXME: set the stack frame
        // processContext的init_stack_frame
        proc.init_stack_frame(entry, stack_top);//初始化进程栈帧
        // FIXME: add to process map
        self.add_proc(pid, proc);
        // FIXME: push to ready queue
        // 将创建好的进程放入队列中
        self.push_ready(pid);
        pid
        //KERNEL_PID
    }
    pub fn get_exit_code(&self,pid:ProcessId) -> Option<isize>{//获取进程的返回值
        if let Some(proc) = self.get_proc(&pid){
            //疑惑：进程退出的判断条件是？
            let proc_inner = proc.read();
            if proc_inner.status() != ProgramStatus::Running{ //如果进程已退出
                let exit_code = proc_inner.exit_code();//获取进程返回值
                return exit_code;
            }
        }
        None//进程还没有退出/get_proc方法返回None
    }

    
    pub fn kill_current(&self, ret: isize) {//杀死当前进程
        self.kill(processor::get_pid(), ret);
    }
    //处理页面错误
    pub fn handle_page_fault(&self, addr: VirtAddr, err_code: PageFaultErrorCode) -> bool {
        // FIXME: handle page fault
        //在 ProcessManager 中，检查缺页异常是否包含越权访问或其他非预期的错误码。
        //如果缺页异常是由于非预期异常导致的，或者缺页异常的地址不在当前进程的栈空间中，直接返回 false。
        //如果缺页异常的地址在当前进程的栈空间中，把缺页异常的处理委托给当前的进程。你可能需要为 ProcessInner 添加用于分配新的栈、更新进程存储信息的函数。
        //在进程的缺页异常处理函数中：分配新的页面、更新页表、更新进程数据中的栈信息。
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
