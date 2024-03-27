//提供创建和管理进程所需的所有信息和方法
use super::*;
use crate::memory::*;
use alloc::sync::Weak;
use alloc::vec::Vec;
use spin::*;
use x86_64::structures::paging::mapper::MapToError;
use x86_64::structures::paging::page::PageRange;
use x86_64::structures::paging::*;
use alloc::sync::Arc;
#[derive(Clone)]
pub struct Process {
    pid: ProcessId,
    inner: Arc<RwLock<ProcessInner>>,
}

pub struct ProcessInner {
    name: String,
    parent: Option<Weak<Process>>,
    children: Vec<Arc<Process>>,
    ticks_passed: usize,
    status: ProgramStatus,
    exit_code: Option<isize>,
    context: ProcessContext,
    page_table: Option<PageTableContext>,
    proc_data: Option<ProcessData>,
}

impl Process {
    #[inline]
    pub fn pid(&self) -> ProcessId {//获取pid
        self.pid
    }

    #[inline]
    pub fn write(&self) -> RwLockWriteGuard<ProcessInner> {//返回类型RwLockWriteGuard<ProcessInner>
        self.inner.write()
    }

    #[inline]
    pub fn read(&self) -> RwLockReadGuard<ProcessInner> {//返回类型RwLockReadGuard<ProcessInner>
        self.inner.read()
    }

    pub fn new(//创建一个新进程，返回Arc<process>
        name: String,
        parent: Option<Weak<Process>>,
        page_table: PageTableContext,
        proc_data: Option<ProcessData>,
    ) -> Arc<Self> {
        let name = name.to_ascii_lowercase();

        // create context
        let pid = ProcessId::new();

        let inner = ProcessInner {
            name,
            parent,
            status: ProgramStatus::Ready,
            context: ProcessContext::default(),
            ticks_passed: 0,
            exit_code: None,
            children: Vec::new(),
            page_table: Some(page_table),
            proc_data: Some(proc_data.unwrap_or_default()),
        };

        trace!("New process {}#{} created.", &inner.name, pid);

        // create process struct
        Arc::new(Self {
            pid,
            inner: Arc::new(RwLock::new(inner)),
        })
    }

    pub fn kill(&self, ret: isize) {//杀死一个进程
        let mut inner = self.inner.write();

        debug!(
            "Killing process {}#{} with ret code: {}",
            inner.name(),
            self.pid,
            ret
        );
        inner.kill(ret);
    }

    pub fn alloc_init_stack(&self) -> VirtAddr {//分配初始栈
        // FIXME: alloc init stack base on self pid

        VirtAddr::new(0)
    }
}

impl ProcessInner {
    pub fn name(&self) -> &str {//获取进程名称
        &self.name
    }

    pub fn tick(&mut self) {//增加进程调度次数
        self.ticks_passed += 1;
    }

    pub fn status(&self) -> ProgramStatus {//返回进程当前状态
        self.status
    }

    pub fn pause(&mut self) {//将进程设置为Ready
        self.status = ProgramStatus::Ready;
    }

    pub fn resume(&mut self) {//将进程设置为Running
        self.status = ProgramStatus::Running;
    }

    pub fn exit_code(&self) -> Option<isize> {//获取进程退出的代码
        self.exit_code
    }

    pub fn clone_page_table(&self) -> PageTableContext {//克隆进程的页表
        self.page_table.as_ref().unwrap().clone_l4()
    }

    pub fn is_ready(&self) -> bool {//检查进程是否为Ready
        self.status == ProgramStatus::Ready
    }

    /// Save the process's context
    /// mark the process as ready
    pub(super) fn save(&mut self, context: &ProcessContext) {//保存进程的上下文
        // FIXME: save the process's context
        self.pause();
        self.context = *context;
    }

    /// Restore the process's context
    /// mark the process as running
    pub(super) fn restore(&mut self, context: &mut ProcessContext) {//恢复进程的上下文
        // FIXME: restore the process's context
        self.resume();
        *context = self.context;//修改传入的可变参数context
        // FIXME: restore the process's page table
        if let Some(page_table) = &self.page_table{
            page_table.load();
        }
    }

    pub fn parent(&self) -> Option<Arc<Process>> {//获取进程的父进程
        self.parent.as_ref().and_then(|p| p.upgrade())
    }

    pub fn kill(&mut self, ret: isize) {//杀死进程
        // FIXME: set exit code

        // FIXME: set status to dead

        // FIXME: take and drop unused resources
    }
}
//实现trait
impl core::ops::Deref for Process {
    type Target = Arc<RwLock<ProcessInner>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl core::ops::Deref for ProcessInner {
    type Target = ProcessData;

    fn deref(&self) -> &Self::Target {
        self.proc_data
            .as_ref()
            .expect("Process data empty. The process may be killed.")
    }
}

impl core::ops::DerefMut for ProcessInner {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.proc_data
            .as_mut()
            .expect("Process data empty. The process may be killed.")
    }
}

impl core::fmt::Debug for Process {//用于格式化进程信息
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let mut f = f.debug_struct("Process");
        f.field("pid", &self.pid);

        let inner = self.inner.read();
        f.field("name", &inner.name);
        f.field("parent", &inner.parent().map(|p| p.pid));
        f.field("status", &inner.status);
        f.field("ticks_passed", &inner.ticks_passed);
        f.field(
            "children",
            &inner.children.iter().map(|c| c.pid.0).collect::<Vec<u16>>(),
        );
        f.field("page_table", &inner.page_table);
        f.field("status", &inner.status);
        f.field("context", &inner.context);
        f.field("stack", &inner.proc_data.as_ref().map(|d| d.stack_segment));
        f.finish()
    }
}

impl core::fmt::Display for Process {//打印进程信息
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let inner = self.inner.read();
        write!(
            f,
            " #{:-3} | #{:-3} | {:12} | {:7} | {:?}",
            self.pid.0,
            inner.parent().map(|p| p.pid.0).unwrap_or(0),
            inner.name,
            inner.ticks_passed,
            inner.status
        )?;
        Ok(())
    }
}
