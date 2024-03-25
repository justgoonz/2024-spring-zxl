//这个文件写测试函数
//打印pid

pub fn test() -> ! {//！表示永远不返回
    let mut count = 0;
    let id;
    if let Some(id_env) = crate::proc::env("id") {//获取进程pid
        id = id_env
    } else {
        id = "unknown".into()//into方法用于在兼容的类型之间转换
    }
    loop {
        // TODO: better way to show more than one process is running?
        count += 1;
        if count == 1000 {
            count = 0;
            print!("\r{:-6} => Tick!", id);//输出pid
        }
        unsafe {
            x86_64::instructions::hlt();//调用halt是cpu进入休眠状态，等待下一个中断
        }
    }
}
//定义一个使用大量栈空间的内联函数
#[inline(never)]
fn huge_stack() {
    println!("Huge stack testing...");
    //分配一个大的栈空间数组
    let mut stack = [0u64; 0x1000];
    //将每个元素设置为索引值
    for (idx, item) in stack.iter_mut().enumerate() {//item是元素的可变引用，idx是索引值
        *item = idx as u64;
    }
    //
    for i in 0..stack.len() / 256 {//从 0 遍历到数组长度除以 256 的结果，步长为1
        println!("{:#05x} == {:#05x}", i * 256, stack[i * 256]);
    }
}

pub fn stack_test() -> ! {
    huge_stack();//测试栈空间能否处理缺页异常
    crate::proc::process_exit(0)//调用进程退出函数，正常退出进程
}
