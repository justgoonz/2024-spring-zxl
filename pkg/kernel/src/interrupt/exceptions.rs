use crate::memory::*;
use x86_64::registers::control::Cr2;
use x86_64::structures::idt::{ InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode };

// see:idt crate
pub unsafe fn register_idt(idt: &mut InterruptDescriptorTable) {
  idt.divide_error.set_handler_fn(divide_error_handler);
  idt.double_fault
    .set_handler_fn(double_fault_handler)
    .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
  idt.page_fault.set_handler_fn(page_fault_handler).set_stack_index(gdt::PAGE_FAULT_IST_INDEX);

  // TODO: you should handle more exceptions here
  // especially gerneral protection fault (GPF)
  // see: https://wiki.osdev.org/Exceptions
}
//异常处理函数，作为参数传递
pub extern "x86-interrupt" fn divide_error_handler(stack_frame: InterruptStackFrame) {
  panic!("EXCEPTION: DIVIDE ERROR\n\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn double_fault_handler(
  stack_frame: InterruptStackFrame,
  error_code: u64
) -> ! {
  panic!("EXCEPTION: DOUBLE FAULT, ERROR_CODE: 0x{:016x}\n\n{:#?}", error_code, stack_frame);
}

pub extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,//保存cpu在中断发生时的寄存器状态
    err_code: PageFaultErrorCode,
) {
    if !crate::proc::handle_page_fault(Cr2::read(), err_code) {//中断处理函数err
        warn!(
            "EXCEPTION: PAGE FAULT, ERROR_CODE: {:?}\n\nTrying to access: {:#x}\n{:#?}",
            err_code,
            Cr2::read(),
            stack_frame
        );
        // FIXME: print info about which process causes page fault?
        let pid = crate::proc::processor::get_pid();
        info!("process {} cause page fault",pid);
        panic!("Cannot handle page fault!");
    }
}
