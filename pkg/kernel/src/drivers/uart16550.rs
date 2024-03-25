use core::fmt;
use x86_64::instructions::port::*;
/// A port-mapped UART 16550 serial interface.
pub struct SerialPort {
    base_port: u16,
    data_port: Port<u8>,                   //A read-write I/O port.
    interrupt_enable_port: Port<u8>,       //中断使能寄存器
    interrupt_fifo_control_port: Port<u8>, //中断识别和FIFO控制寄存器
    line_control_port: Port<u8>,           // 线路控制寄存器
    modem_control_port: Port<u8>,          // 调制解调器控制寄存器
    line_status_port: Port<u8>,            // 线路状态寄存器
    modem_status_port: Port<u8>,           // 调制解调器mak状态寄存器
    scratch_port: Port<u8>,                // Scratch 寄存器
}
//使用::来调用模块，类型的方法
//使用.来调用某个实例的方法
impl SerialPort {
    pub const fn new(port: u16) -> Self {
        Self {
            base_port: port,
            data_port: Port::new(port),
            interrupt_enable_port: Port::new(port + 1), // DATA/Divisor Latch and Interrupt Enable
            interrupt_fifo_control_port: Port::new(port + 2), // FIFO Control/Interrupt Identification
            line_control_port: Port::new(port + 3),           // Line Control
            modem_control_port: Port::new(port + 4),          // Modem Control
            line_status_port: Port::new(port + 5),            // Line Status
            modem_status_port: Port::new(port + 6),           // Modem Status
            scratch_port: Port::new(port + 7),                // Scratch
        }
    }

    /// Initializes the serial port.
    pub fn init(&mut self) -> Result<(), &'static str> {
        unsafe {
            // Disable all interrupts
            self.interrupt_enable_port.write(0x00);

            // Enable DLAB (set baud rate divisor)
            self.line_control_port.write(0x80);

            // Set divisor to 3 (lo byte) 38400 baud
            self.data_port.write(0x03);
            // Set divisor (hi byte)
            self.interrupt_enable_port.write(0x00);

            // 8 bits, no parity, one stop bit
            self.line_control_port.write(0x03);

            // Enable FIFO, clear them, with 14-byte threshold
            self.interrupt_fifo_control_port.write(0xC7);

            // IRQs enabled, RTS/DSR set
            self.modem_control_port.write(0x0B);

            // Optional: Depends on specific use case
            self.modem_control_port.write(0x1E);

            self.data_port.write(0xAE); // Test serial 
            // test
            if self.data_port.read() != 0xAE {
                return Err("Failed to initialize serial port: Self-test failed");
            }

            self.modem_control_port.write(0x0F); 

            self.interrupt_enable_port.write(0x01); // Enable interrupt
        }
        Ok(())
    }

    /// Sends a byte on the serial port.
    pub fn send(&mut self, data: u8) {
        //这里有没有优化的空间，在单线程的情况下，while可能会导致阻塞
        while !self.is_transmit_empty() { 
             // wait until the transmitter is ready
        }
        unsafe {
            self.data_port.write(data);
            //self.data_port.write(0x80);
        }
    }

    fn is_transmit_empty(&mut self) -> bool {
        unsafe {
            //若传输寄存器为空，串行端口开始发送新数据
            (self.line_status_port.read() & 0x20) != 0
        }
    }

    /// Receives a byte on the serial port no wait.
    pub fn receive(&mut self) -> Option<u8> {//串口读
        while !self.is_receive_ready() { //中断所导致的“并发访问”是强制性的，并且需要主动恢复，循环等待的过程并不存在抢占。
            // wait 
        } 
        unsafe { Some(self.data_port.read()) }
    }
    fn is_receive_ready(&mut self) -> bool {
        unsafe { (self.line_status_port.read() & 0x1) != 0 }
    }
}

impl fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.send(byte);
        }
        Ok(())
    }
}
