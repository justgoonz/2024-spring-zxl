use x86_64::structures::idt::{ InterruptDescriptorTable, InterruptStackFrame };
use super::consts::*;
use crate::drivers::input;
use crate::drivers::serial::get_serial_for_sure;
//once_mutex!(pub SERIAL: SerialPort);
//use crate::drivers::input::
pub unsafe fn register_idt(idt: &mut InterruptDescriptorTable) {
  //serial0改成timer就可以中断了，否则进入不了中断处理函数
  idt[(Interrupts::IrqBase as usize) + (Irq::Serial0 as usize)].set_handler_fn(serial_handler);
}

pub extern "x86-interrupt" fn serial_handler(_st: InterruptStackFrame) {
  //中断处理函数
  //println!("interrupted of serail_handler");
  receive();
  super::ack();
}

/*lazy_static! { //Mutex互斥锁
  static ref SERIAL_PORT: Mutex<SerialPort> = Mutex::new(SerialPort::new(0x3f8));
}*/
/// Receive character from uart 16550
/// Should be called on every interrupt

#[inline]
fn receive() {
  //从串口读取数据到缓冲区
  // FIXME: receive character from uart 16550, put it into INPUT_BUFFER
  let mut serial = get_serial_for_sure();
  let ch = serial.receive();
  drop(serial);//释放
  if let Some(ch) = ch  {
    //调用串口
    input::push_key(ch); //将数据放入缓冲区
  }
}
