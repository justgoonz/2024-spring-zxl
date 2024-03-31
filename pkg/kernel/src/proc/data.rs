//定义和管理进程的内存布局
use alloc::{collections::BTreeMap, sync::Arc};
use spin::RwLock;
use x86_64::structures::paging::{
    page::{PageRange, PageRangeInclusive},
    Page,
};

use super::*;

#[derive(Debug, Clone)]
pub struct ProcessData {
    // shared data
    pub(super) env: Arc<RwLock<BTreeMap<String, String>>>,//线程安全的引用计数和读写保护，键值对映射的方式存储环境变量

    // process specific data
    pub(super) stack_segment: Option<PageRange>//线程栈段的页面
}

impl Default for ProcessData {
    fn default() -> Self {
        Self {
            env: Arc::new(RwLock::new(BTreeMap::new())),
            stack_segment: None
        }
    }
}

impl ProcessData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn env(&self, key: &str) -> Option<String> {//获取环境变量
        self.env.read().get(key).cloned()
    }

    pub fn set_env(&mut self, key: &str, val: &str) {//传递一个键值对，设置环境变量
        self.env.write().insert(key.into(), val.into());
    }

    pub fn set_stack(&mut self, start: VirtAddr, size: u64) {
        let start = Page::containing_address(start);//分配一个页面
        self.stack_segment = Some(Page::range(start, start + size));
    }

    pub fn is_on_stack(&self, addr: VirtAddr) -> bool {//检查地址是否在栈上
        // FIXME: check if the address is on the stack
        // 未实现
        let masked_addr =addr.as_u64() & STACK_START_MASK;
        false
    }
}
