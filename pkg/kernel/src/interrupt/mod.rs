mod apic;
mod consts;
pub mod clock;
mod serial;
mod exceptions;

use apic::*;
use x86_64::structures::idt::InterruptDescriptorTable;
use crate::memory::physical_to_virtual;


lazy_static! {
  //ref表示引用对象，和&类似，前者用于申请返回引用，后者用于主动返回引用
  static ref IDT: InterruptDescriptorTable = {
    let mut idt = InterruptDescriptorTable::new();
    unsafe {
      exceptions::register_idt(&mut idt); //注册中断描述符表
      clock::register_idt(&mut idt);
      serial::register_idt(&mut idt);
    }
    idt
  };
}

/// init interrupts system
pub fn init() {
  IDT.load();

  // FIXME: check and init APIC
  if XApic::support() {
    let mut lapic = unsafe { XApic::new(physical_to_virtual(LAPIC_ADDR)) };
    lapic.cpu_init();
    info!("APIC Initialized.");
  } else {
    panic!("APIC not supported!");
  }
  // FIXME: enable serial irq with IO APIC (use enable_irq)
  //enable_irq(consts::Interrupts::IrqBase as u8 + consts::Irq::Keyboard as u8,0);
  enable_irq(consts::Irq::Serial0 as u8,0);//这里是irq序号不是中断号
  info!("Serial IRQ Enabled and Interrupts Initialized.");
  
  info!("Interrupts Initialized.");
}

#[inline(always)]
pub fn enable_irq(irq: u8, cpuid: u8) {
  let mut ioapic = unsafe { IoApic::new(physical_to_virtual(IOAPIC_ADDR)) };
  ioapic.enable(irq, cpuid);
}

#[inline(always)]
pub fn ack() {
  let mut lapic = unsafe { XApic::new(physical_to_virtual(LAPIC_ADDR)) };
  lapic.eoi();
}
