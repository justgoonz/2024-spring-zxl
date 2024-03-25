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
//中断处理函数，作为参数传递
pub extern "x86-interrupt" fn divide_error_handler(stack_frame: InterruptStackFrame) {
  panic!("EXCEPTION: DIVIDE ERROR\n\n{:#?}", stack_frame);
}
//双重故障是不可恢复错误，声明-> !告诉编译器这个函数不会返回
pub extern "x86-interrupt" fn double_fault_handler(
  stack_frame: InterruptStackFrame,
  error_code: u64
) -> ! {
  panic!("EXCEPTION: DOUBLE FAULT, ERROR_CODE: 0x{:016x}\n\n{:#?}", error_code, stack_frame);
}
//这些是不可恢复错误，因此不进行错误处理，直接panic!
pub extern "x86-interrupt" fn page_fault_handler(
  stack_frame: InterruptStackFrame,
  err_code: PageFaultErrorCode
) {
  panic!(
    "EXCEPTION: PAGE FAULT, ERROR_CODE: {:?}\n\nTrying to access: {:#x}\n{:#?}",
    err_code,
    Cr2::read(),
    stack_frame
  );
}
