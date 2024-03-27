//页表
use crate::memory::*;
use core::ptr::copy_nonoverlapping;

use alloc::sync::Arc;
use x86_64::{
    registers::control::{Cr3, Cr3Flags},
    structures::paging::*,
    VirtAddr,
};

pub struct Cr3RegValue {//CR3寄存器
    pub addr: PhysFrame,
    pub flags: Cr3Flags,
}

impl Cr3RegValue {
    pub fn new(addr: PhysFrame, flags: Cr3Flags) -> Self {
        Self { addr, flags }
    }
}

pub struct PageTableContext {//页表上下文(区别于进程的上下文，将新的页表加载到CR3寄存器时，要处理页表的上下文)
    pub reg: Arc<Cr3RegValue>,//CR3寄存器的值
}

impl PageTableContext {//页表上下文的方法
    pub fn new() -> Self {//创建一个页表对象
        let (frame, flags) = Cr3::read();
        Self {
            reg: Arc::new(Cr3RegValue::new(frame, flags)),
        }
    }

    /// Create a new page table object based on current page table.
    pub fn clone_l4(&self) -> Self {//根据当前页表创建一个新的页表对象
        // 1. alloc new page table
        let mut frame_alloc = crate::memory::get_frame_alloc_for_sure();
        let page_table_addr = frame_alloc
            .allocate_frame()
            .expect("Cannot alloc page table for new process.");

        // 2. copy current page table to new page table
        unsafe {
            copy_nonoverlapping::<PageTable>(
                physical_to_virtual(self.reg.addr.start_address().as_u64()) as *mut PageTable,
                physical_to_virtual(page_table_addr.start_address().as_u64()) as *mut PageTable,
                1,
            );
        }

        // 3. create page table object
        Self {
            reg: Arc::new(Cr3RegValue::new(page_table_addr, Cr3Flags::empty())),
        }
    }

    /// Load the page table to Cr3 register.
    pub fn load(&self) {//加载页表到CR3寄存器
        unsafe { Cr3::write(self.reg.addr, self.reg.flags) }
    }

    /// Get the page table object by Cr3 register value.
    pub fn mapper(&self) -> OffsetPageTable<'static> {//获取页表
        unsafe {
            OffsetPageTable::new(
                (physical_to_virtual(self.reg.addr.start_address().as_u64()) as *mut PageTable)
                    .as_mut()
                    .unwrap(),
                VirtAddr::new_truncate(*PHYSICAL_OFFSET.get().unwrap()),
            )
        }
    }
}

impl core::fmt::Debug for PageTableContext {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PageTable")
            .field("addr", &self.reg.addr)
            .field("flags", &self.reg.flags)
            .finish()
    }
}
