use super::LocalApic;
use crate::interrupt::consts::{ Interrupts, Irq };
use bit_field::BitField;
use core::fmt::{ Debug, Error, Formatter };
use core::ptr::{ read_volatile, write_volatile };
use x86::cpuid::CpuId; //crate为离当前路径最近的.toml文件所在的路径
use crate::memory::address;
/// Default physical address of xAPIC
pub const LAPIC_ADDR: u64 = 0xfee00000;

pub struct XApic {
  addr: u64,
}
//实现对xapic寄存器的读写操作
impl XApic {
  pub unsafe fn new(addr: u64) -> Self {
    XApic{addr:addr}
  }

  unsafe fn read(&self, reg: u32) -> u32 {

    read_volatile((self.addr + (reg as u64)) as *const u32) //原子读
  }

  unsafe fn write(&mut self, reg: u32, value: u32) {
    write_volatile((self.addr + (reg as u64)) as *mut u32, value); //原子写
    self.read(0x20); //返回设备寄存器的状态，这和写操作是否成功有关
  }
}

impl LocalApic for XApic {
  /// If this type APIC is supported
  fn support() -> bool {
    // FIXME: Check CPUID to see if xAPIC is supported.
    CpuId::new()
      .get_feature_info()
      .map(|f| f.has_apic())
      .unwrap_or(false) //如果对象是一个some,则直接返回，如果对象是一个none，则返回参数值
  }

  /// Initialize the xAPIC for the current CPU.
  /// 规范文档：https://wiki.osdev.org/APIC
  fn cpu_init(&mut self) {
    unsafe {
      // FIXME: Enable local APIC; set spurious interrupt vector.
      let mut spiv = self.read(0xf0); //spic寄存器号0xf0
      spiv |= 1 << 8; // set EN bit
      // clear and set Vector
      spiv &= !0xff; //清空第八位
      spiv |= (Interrupts::IrqBase as u32) + (Irq::Spurious as u32);
      self.write(0xf0, spiv);
      // FIXME: The timer repeatedly counts down at bus frequency
      let mut lvt_timer = self.read(0x320);
      // clear and set Vector
      lvt_timer &= !0xff;
      lvt_timer |= (Interrupts::IrqBase as u32) + (Irq::Timer as u32);
      lvt_timer &= !(1 << 16); // clear Mask
      lvt_timer |= 1 << 17; // set Timer Periodic Mode
      self.write(0x320, lvt_timer);
      // FIXME: Disable logical interrupt lines (LINT0, LINT1)
      self.write(0x350, 1 << 16); // set Mask
      self.write(0x360, 1 << 16);
      // FIXME: Disable performance counter overflow interrupts (PCINT)
      self.write(0x340, 1 << 16);
      // FIXME: Map error interrupt to IRQ_ERROR.
      let mut error_vector = self.read(0x370);
      error_vector &= !0xff; // Clear existing vector
      error_vector |= Irq::Error as u32; // Assuming Irq::Error is the error interrupt vector
      self.write(0x370, error_vector);
      // FIXME: Clear error status register (requires back-to-back writes).
      self.write(0x280, 0);
      self.write(0x280, 0);
      // FIXME: Ack any outstanding interrupts.
      self.eoi(); //通知xpic中断处理状态，0表示中断处理已完成，非0表示正在中断
      // FIXME: Send an Init Level De-Assert to synchronise arbitration ID's.
      self.write(0x310, 0); // set ICR 0x310
      const BCAST: u32 = 1 << 19;
      const INIT: u32 = 5 << 8;
      const TMLV: u32 = 1 << 15; // TM = 1, LV = 0
      self.write(0x300, BCAST | INIT | TMLV); // set ICR 0x300
      const DS: u32 = 1 << 12;
      while (self.read(0x300) & DS) != 0 {} // wait for delivery status
      // FIXME: Enable interrupts on the APIC (but not on the processor).
      self.write(0x80, 0); // TPR register is at offset 0x80
    }

    // NOTE: Try to use bitflags! macro to set the flags.
  }

  fn id(&self) -> u32 {
    // NOTE: Maybe you can handle regs like `0x0300` as a const.
    unsafe {
      self.read(0x0020) >> 24
    }
  }

  fn version(&self) -> u32 {
    unsafe { self.read(0x0030) }
  }

  fn icr(&self) -> u64 {
    unsafe { ((self.read(0x0310) as u64) << 32) | (self.read(0x0300) as u64) }
  }

  fn set_icr(&mut self, value: u64) {
    unsafe {
      while self.read(0x0300).get_bit(12) {}
      self.write(0x0310, (value >> 32) as u32);
      self.write(0x0300, value as u32);
      while self.read(0x0300).get_bit(12) {}
    }
  }

  fn eoi(&mut self) {
    unsafe {
      self.write(0x00b0, 0);
    }
  }
}

impl Debug for XApic {
  fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
    f.debug_struct("Xapic")
      .field("id", &self.id())
      .field("version", &self.version())
      .field("icr", &self.icr())
      .finish()
  }
}