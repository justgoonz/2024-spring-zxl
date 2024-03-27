use super::consts::*;
use core::sync::atomic::{AtomicU64, Ordering};
use x86_64::structures::idt::{InterruptDescriptorTable,InterruptStackFrame};
use crate::{memory::gdt, proc::switch};
pub unsafe fn register_idt(idt: &mut InterruptDescriptorTable) {
    idt[Interrupts::IrqBase as usize + Irq::Timer as usize]
        .set_handler_fn(process_scheduler_handler)
        .set_stack_index(gdt::TIMER_IST_INDEX);
}
as_handler!(process_scheduler);

pub extern "x86-interrupt" fn process_scheduler(_sf: InterruptStackFrame) {//idt负责传递这个参数
    //FIXME
    x86_64::instructions::interrupts::without_interrupts(|| {//在中断关闭的状态下继续执行，保证操作的原子性，防止被其他中断打断
    // do something
})
}

static COUNTER: AtomicU64 = AtomicU64::new(0);

#[inline]
pub fn read_counter() -> u64 {
    // FIXME: load counter value
    COUNTER.load(Ordering::Relaxed)//Loads a value from the atomic integer.
}

#[inline]
pub fn inc_counter() -> u64 {
    // FIXME: read counter value and increase it
    COUNTER.fetch_add(1, Ordering::Relaxed)//Adds to the current value, returning the previous value.
}