#![no_std] //编写裸机程序，因此禁用标准库
#![no_main]//指定标准入口不是main函数，适用于系统级程序
#![feature(alloc_error_handler)]

#[macro_use]
extern crate log;
extern crate alloc;


//mod lib 导致编译器报错:lib
use elf::*;

use alloc::boxed::Box;
use alloc::vec;
use uefi::prelude::*;
use x86_64::registers::control::*;
use x86_64::VirtAddr;
use ysos_boot::*;
use xmas_elf::ElfFile;

// path of config file
const CONFIG_PATH: &str = "\\EFI\\BOOT\\boot.conf";
// uefi程序入口点
#[entry]
fn efi_main(image: uefi::Handle, mut system_table: SystemTable<Boot>) -> Status {
    //init
    uefi_services::init(&mut system_table).expect("Failed to initialize utilities");
    //log level
    log::set_max_level(log::LevelFilter::Info);
    info!("Running UEFI bootloader...");

    let bs = system_table.boot_services();

    // 1. Load config
    //let config = { /* FIXME: Load config file */ };
    //open file
    let mut config_file = open_file(&bs, CONFIG_PATH);
    //load file
    let config_data = load_file(&bs, &mut config_file);
    let config = ysos_boot::config::Config::parse(config_data);
    
    info!("Config: {:#x?}", config);//日志消息格式，用于输出结构体的信息

    // 2. Load ELF files
    //let elf = { /* FIXME: Load kernel elf file */ };
    let mut elf_file = open_file(&bs, &config.kernel_path);
    let elf_data = load_file(&bs,&mut elf_file);
    let elf = ElfFile::new(elf_data).expect("fialed to new an ElfFile");
    unsafe {
        set_entry(elf.header.pt2.entry_point() as usize);
    }

    // 3. Load MemoryMap
    let max_mmap_size = system_table.boot_services().memory_map_size();
    let mmap_storage = Box::leak(//Box::leak 是一个将 Box 转换为裸指针并泄漏其内存的方法，防止 Rust 的自动内存回收
        vec![0; max_mmap_size.map_size + 10 * max_mmap_size.entry_size].into_boxed_slice(),
    );
    let mmap = system_table
        .boot_services()
        .memory_map(mmap_storage)
        .expect("Failed to get memory map");

    let max_phys_addr = mmap
        .entries()
        .map(|m| m.phys_start + m.page_count * 0x1000)
        .max()
        .unwrap()
        .max(0x1_0000_0000); // include IOAPIC MMIO area

    // 4. Map ELF segments, kernel stack and physical memory to virtual memory
    let mut page_table = current_page_table();

    // FIXME: root page table is readonly, disable write protect (Cr0)
    unsafe{
        Cr0::update(|f|{//关闭根页表的写保护，在启动程序结束后，需要恢复
            f.remove(Cr0Flags::WRITE_PROTECT);
        })
    }
    // FIXME: map physical memory to specific virtual address offset
    //调用map_physical_memory
    // 物理内存映射到虚拟地址
    let mut frame_allocator = UEFIFrameAllocator(bs);
    map_physical_memory(
        config.physical_memory_offset,
        max_phys_addr,
        &mut page_table,
        &mut frame_allocator,
    );

    // FIXME: load and map the kernel elf file
    //调用load_elf
    // 加载并映射内核 ELF 文件
    load_elf(
        &elf, // 已加载的 ELF 文件
        config.physical_memory_offset, // 物理地址偏移量
        &mut page_table, // 页表映射器
        &mut frame_allocator, // 物理帧分配器
    ).expect("Failed to load elf");

    // FIXME: map kernel stack
    // 映射内核栈
    let stack_end = VirtAddr::new(config.kernel_stack_address + config.kernel_stack_size * 0x1000);
    let stack_start = VirtAddr::new(config.kernel_stack_address);
    map_range(
        stack_start.as_u64(),
        (stack_end - stack_start) / 0x1000, // 计算栈的大小，单位是页
        &mut page_table, // 页表映射器
        &mut frame_allocator, // 物理帧分配器
    ).expect("Failed to map kernel stack by map_range");

    // FIXME: recover write protect (Cr0)
    unsafe{
        Cr0::update(|f|{//关闭根页表的写保护，在启动程序结束后，需要恢复
            f.insert(Cr0Flags::WRITE_PROTECT);
        })
    }
    free_elf(bs, elf);

    // 5. Exit boot and jump to ELF entry
    info!("Exiting boot services...");

    let (runtime, mmap) = system_table.exit_boot_services(MemoryType::LOADER_DATA);
    // NOTE: alloc & log are no longer available

    // construct BootInfo
    let bootinfo = BootInfo {
        memory_map: mmap.entries().copied().collect(),
        physical_memory_offset: config.physical_memory_offset,
        system_table: runtime,
    };

    // align stack to 8 bytes
    let stacktop = config.kernel_stack_address + config.kernel_stack_size * 0x1000 - 8;

    unsafe {
        jump_to_entry(&bootinfo, stacktop);
    }
}
