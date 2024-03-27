//gdt.rs：定义 TSS 和 GDT，为内核提供内存段描述符和任务状态段。
use lazy_static::lazy_static;
use x86_64::registers::segmentation::Segment;
use x86_64::structures::gdt::{ Descriptor, GlobalDescriptorTable, SegmentSelector };
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0; //interupt stack table
pub const PAGE_FAULT_IST_INDEX: u16 = 1;
pub const TIMER_IST_INDEX :u16 =2;

pub const IST_SIZES: [usize; 4] = [0x1000, 0x1000, 0x1000, 0x1000]; //常量数组，表示内存块的大小

/* pub struct TaskStateSegment {
    reserved_1: u32,
    pub privilege_stack_table: [VirtAddr; 3],//特权栈
    reserved_2: u64,
    pub interrupt_stack_table: [VirtAddr; 7],//中断栈
    reserved_3: u64,
    reserved_4: u16,
    pub iomap_base: u16,
}*/

lazy_static! {
  static ref TSS: TaskStateSegment = {
    //任务状态段
    let mut tss = TaskStateSegment::new();

    // initialize the TSS with the static buffers
    // will be allocated on the bss section when the kernel is load
    //
    // DO NOT MODIFY THE FOLLOWING CODE
    tss.privilege_stack_table[0] = {
      //计算虚拟地址
      const STACK_SIZE: usize = IST_SIZES[0];
      static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE]; // empty stack
      let stack_start = VirtAddr::from_ptr(unsafe { STACK.as_ptr() }); //Creates a virtual address from the given pointer
      let stack_end = stack_start + STACK_SIZE;
      info!("Privilege Stack  : 0x{:016x}-0x{:016x}", stack_start.as_u64(), stack_end.as_u64());
      stack_end
    };
    // FIXME: fill tss.interrupt_stack_table with the static stack buffers like above
    // You can use `tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize]`
    tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
      //usize类型用来表示索引下标
      const STACK_SIZE: usize = IST_SIZES[1];
      static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
      let stack_start = VirtAddr::from_ptr(unsafe { STACK.as_ptr() });
      let stack_end = stack_start + STACK_SIZE;
      info!("Double Fault Stack: 0x{:016x}-0x{:016x}", stack_start.as_u64(), stack_end.as_u64());
      stack_end
    };
    tss.interrupt_stack_table[PAGE_FAULT_IST_INDEX as usize] = {
      const STACK_SIZE: usize = IST_SIZES[2];
      static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
      let stack_start = VirtAddr::from_ptr(unsafe { &STACK.as_ptr() });
      let stack_end = stack_start + STACK_SIZE;
      info!("Page Fault Stack : 0x{:016x}-0x{:016x}", stack_start.as_u64(), stack_end.as_u64());
      stack_end
    };
    //FIXME:声明一个中断栈
    tss.interrupt_stack_table[TIMER_IST_INDEX as usize]={
      const STACK_SIZE: usize = IST_SIZES[3];
      static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
      let stack_start = VirtAddr::from_ptr(unsafe { &STACK.as_ptr() });//from_ptr方法接受一个物理地址，根据该地址分配一个虚拟地址
      let stack_end = stack_start + STACK_SIZE;
      info!("Timer Stack : 0x{:016x}-0x{:016x}", stack_start.as_u64(), stack_end.as_u64());
      stack_end
    };
      tss
    };
  }


lazy_static! {
  static ref GDT: (GlobalDescriptorTable, KernelSelectors) = {
    let mut gdt = GlobalDescriptorTable::new();
    let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
    let data_selector = gdt.add_entry(Descriptor::kernel_data_segment());
    let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
    (
      gdt,
      KernelSelectors {
        code_selector,
        data_selector,
        tss_selector,
      },
    )
  };
}

#[derive(Debug)]
pub struct KernelSelectors {
  pub code_selector: SegmentSelector,
  pub data_selector: SegmentSelector,
  tss_selector: SegmentSelector,
}

pub fn init() {
  use x86_64::instructions::segmentation::{ CS, DS, ES, FS, GS, SS };
  use x86_64::instructions::tables::load_tss;
  use x86_64::PrivilegeLevel;

  GDT.0.load();
  unsafe {
    CS::set_reg(GDT.1.code_selector);
    DS::set_reg(GDT.1.data_selector);
    SS::set_reg(SegmentSelector::new(0, PrivilegeLevel::Ring0));
    ES::set_reg(SegmentSelector::new(0, PrivilegeLevel::Ring0));
    FS::set_reg(SegmentSelector::new(0, PrivilegeLevel::Ring0));
    GS::set_reg(SegmentSelector::new(0, PrivilegeLevel::Ring0));
    load_tss(GDT.1.tss_selector);
  }

  let mut size = 0;

  for &s in IST_SIZES.iter() {
    size += s;
  }

  let (size, unit) = crate::humanized_size(size as u64);
  info!("Kernel IST Size  : {:>7.*} {}", 3, size, unit);

  info!("GDT Initialized.");
}

pub fn get_selector() -> &'static KernelSelectors {
  &GDT.1
}
