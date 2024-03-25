use uefi::proto::media::file::*;
use uefi::proto::media::fs::SimpleFileSystem;
use uefi::table::boot::*;
use xmas_elf::ElfFile;

/// Open root directory
/// 文件系统地入口
pub fn open_root(bs: &BootServices) -> Directory {
    let handle = bs
        .get_handle_for_protocol::<SimpleFileSystem>() //获取文件系统地句柄
        .expect("Failed to get handle for SimpleFileSystem");

    let fs = bs
        .open_protocol_exclusive::<SimpleFileSystem>(handle) //独占地方式打开文件系统
        .expect("Failed to get FileSystem");
    let mut fs = fs;

    fs.open_volume().expect("Failed to open volume")
}

/// Open file at `path`
pub fn open_file(bs: &BootServices, path: &str) -> RegularFile {
    let mut buf = [0; 64];//初始话为0，大小为64
    //'from_str_with_buf' : Convert a &str to a &CStr16, backed by a buffer.
    let cstr_path = uefi::CStr16::from_str_with_buf(path, &mut buf).unwrap();

    let handle = open_root(bs)
        .open(cstr_path, FileMode::Read, FileAttribute::empty())//‘Attribute’属性
        .expect("Failed to open file");

    match handle.into_type().expect("Failed to into_type") { //若打开地是一个目录，或无效路径，则panic!
        FileType::Regular(regular) => regular,
        _ => panic!("Invalid file type"),
    }
}

/// Load file to new allocated pages
pub fn load_file(bs: &BootServices, file: &mut RegularFile) -> &'static mut [u8] {
    let mut info_buf = [0u8; 0x100];
    let info = file
        .get_info::<FileInfo>(&mut info_buf)
        .expect("Failed to get file info");

    let pages = info.file_size() as usize / 0x1000 + 1;

    let mem_start = bs
        .allocate_pages(AllocateType::AnyPages, MemoryType::LOADER_DATA, pages)
        .expect("Failed to allocate pages");
    //根据指针和长度读取内存形成slice
    let buf = unsafe { core::slice::from_raw_parts_mut(mem_start as *mut u8, pages * 0x1000) };
    let len = file.read(buf).expect("Failed to read file");

    info!(
        "Load file \"{}\" to memory, size = {}",
        info.file_name(),
        len
    );

    &mut buf[..len]//文件内容的引用
}

/// Free ELF files for which the buffer was created using 'load_file'
pub fn free_elf(bs: &BootServices, elf: ElfFile) {
    let buffer = elf.input;//文件内容的引用
    let pages = buffer.len() / 0x1000 + 1;
    let mem_start = buffer.as_ptr() as u64;

    unsafe {
        bs.free_pages(mem_start, pages).expect("Failed to free pages");
    }
}
