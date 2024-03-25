use uefi::table::boot::*;
use x86_64::{structures::paging::*, PhysAddr};

/// Use `BootServices::allocate_pages()` as frame allocator
// 帧分配器结构体
pub struct UEFIFrameAllocator<'a>(pub &'a BootServices);

unsafe impl FrameAllocator<Size4KiB> for UEFIFrameAllocator<'_> {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        // 分配一个页，返回物理地址
        let addr = self
            .0
            .allocate_pages(AllocateType::AnyPages, MemoryType::LOADER_DATA, 1)
            .expect("Failed to allocate frame");
        // 根据物理地址，创建物理帧对象
        let frame = PhysFrame::containing_address(PhysAddr::new(addr));
        Some(frame)
    }
}
