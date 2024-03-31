use volatile::{access::ReadOnly, VolatileRef};
use x86_64::{registers::rflags::RFlags, structures::idt::InterruptStackFrameValue, VirtAddr};

use crate::{memory::gdt::get_selector, RegistersValue};

#[repr(C)]//按照c语言规则内存布局
#[derive(Clone, Copy)]//自动实现括号中的trait
pub struct ProcessContextValue {//表示进程状态
    pub regs: RegistersValue,//寄存器
    pub stack_frame: InterruptStackFrameValue,//中断栈帧
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct ProcessContext {
    value: ProcessContextValue,
}

impl ProcessContext {
    #[inline]
    pub fn as_mut(&mut self) -> VolatileRef<ProcessContextValue> {//返回可变引用，VolatileRef只允许同一时间一个线程访问
        VolatileRef::from_mut_ref(&mut self.value)
    }

    #[inline]
    pub fn as_ref(&self) -> VolatileRef<'_, ProcessContextValue, ReadOnly> {//获取只读引用
        VolatileRef::from_ref(&self.value)
    }

    #[inline]
    pub fn set_rax(&mut self, value: usize) {//设置寄存器RAX
        self.value.regs.rax = value;
    }

    #[inline]
    pub fn save(&mut self, context: &ProcessContext) {//保存当前进程的上下文
        self.value = context.as_ref().as_ptr().read();
    }

    #[inline]
    pub fn restore(&self, context: &mut ProcessContext) {//恢复当前进程的上下文
        context.as_mut().as_mut_ptr().write(self.value);
    }
    // 初始化进程的栈帧，设置入口点地址和栈顶地址
    pub fn init_stack_frame(&mut self, entry: VirtAddr, stack_top: VirtAddr) {
        self.value.stack_frame.stack_pointer = stack_top;
        self.value.stack_frame.instruction_pointer = entry;
        self.value.stack_frame.cpu_flags =//设置cpu标志
            (RFlags::IOPL_HIGH | RFlags::IOPL_LOW | RFlags::INTERRUPT_FLAG).bits();

        let selector = get_selector();//获取gdt选择器
        self.value.stack_frame.code_segment = selector.code_selector.0 as u64;
        self.value.stack_frame.stack_segment = selector.data_selector.0 as u64;

        trace!("Init stack frame: {:#?}", &self.stack_frame);
    }
}

impl Default for ProcessContextValue {//默认实现
    fn default() -> Self {
        Self {
            regs: RegistersValue::default(),
            stack_frame: InterruptStackFrameValue {
                instruction_pointer: VirtAddr::new_truncate(0),
                code_segment: 8,
                cpu_flags: 0,
                stack_pointer: VirtAddr::new_truncate(0),
                stack_segment: 0,
            },
        }
    }
}

impl core::ops::Deref for ProcessContext {
    type Target = ProcessContextValue;

    #[inline]
    fn deref(&self) -> &Self::Target {//允许用过引用直接访问内部类型ProcessContextValue
        &self.value
    }
}

impl core::fmt::Debug for ProcessContext {//用于调试输出
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        self.value.fmt(f)
    }
}

impl core::fmt::Debug for ProcessContextValue {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {//返回调试信息
        let mut f = f.debug_struct("StackFrame");
        f.field("stack_top", &self.stack_frame.stack_pointer);
        f.field("cpu_flags", &self.stack_frame.cpu_flags);
        f.field("instruction_pointer", &self.stack_frame.instruction_pointer);
        f.field("regs", &self.regs);
        f.finish()
    }
}
