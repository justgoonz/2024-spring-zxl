use core::fmt;

#[repr(align(8), C)]
#[derive(Clone, Default, Copy)]
pub struct RegistersValue {//x86寄存器集合，每个字段代表一个寄存器
    //r15-r8 通用寄存器
    pub r15: usize,
    pub r14: usize,
    pub r13: usize,
    pub r12: usize,
    pub r11: usize,
    pub r10: usize,
    pub r9: usize,
    pub r8: usize,
    pub rdi: usize,//destination index
    pub rsi: usize,//source index
    pub rdx: usize,//data
    pub rcx: usize,//count
    pub rbx: usize,//base
    pub rax: usize,//accumulator
    pub rbp: usize,//base pointer
}

impl fmt::Debug for RegistersValue {//以调试格式输出寄存器值
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Registers")?;
        write!(f, "r15: 0x{:016x}, ", self.r15)?;
        write!(f, "r14: 0x{:016x}, ", self.r14)?;
        writeln!(f, "r13: 0x{:016x},", self.r13)?;
        write!(f, "r12: 0x{:016x}, ", self.r12)?;
        write!(f, "r11: 0x{:016x}, ", self.r11)?;
        writeln!(f, "r10: 0x{:016x},", self.r10)?;
        write!(f, "r9 : 0x{:016x}, ", self.r9)?;
        write!(f, "r8 : 0x{:016x}, ", self.r8)?;
        writeln!(f, "rdi: 0x{:016x},", self.rdi)?;
        write!(f, "rsi: 0x{:016x}, ", self.rsi)?;
        write!(f, "rdx: 0x{:016x}, ", self.rdx)?;
        writeln!(f, "rcx: 0x{:016x},", self.rcx)?;
        write!(f, "rbx: 0x{:016x}, ", self.rbx)?;
        write!(f, "rax: 0x{:016x}, ", self.rax)?;
        write!(f, "rbp: 0x{:016x}", self.rbp)?;
        Ok(())
    }
}

#[macro_export]//允许在crate外使用
macro_rules! as_handler {
    ($fn: ident) => {//接受一个标识符fn作为参数
        paste::item! {//用于拼接标识符的宏
            #[naked]//裸函数：这个函数本身负责保护和恢复寄存器状态，不允许编译器干预
            pub extern "x86-interrupt" fn [<$fn _handler>](_sf: InterruptStackFrame) {//宏生成fn_handler的新函数名
                unsafe {
                    //内联汇编宏 1.将当前cpu寄存器值压入栈中，保存中断处理期间cpu状态
                    //2.调用实际的中断处理函数，{}传入中断处理函数名
                    //3.将寄存器的值弹出栈
                    core::arch::asm!("
                    push rbp
                    push rax
                    push rbx
                    push rcx
                    push rdx
                    push rsi
                    push rdi
                    push r8
                    push r9
                    push r10
                    push r11
                    push r12
                    push r13
                    push r14
                    push r15
                    call {}
                    pop r15
                    pop r14
                    pop r13
                    pop r12
                    pop r11
                    pop r10
                    pop r9
                    pop r8
                    pop rdi
                    pop rsi
                    pop rdx
                    pop rcx
                    pop rbx
                    pop rax
                    pop rbp
                    iretq",
                    sym $fn, options(noreturn));
                }
            }
        }
    };
}
