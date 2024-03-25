use crossbeam_queue::ArrayQueue;
use heapless::String; // 使用heapless的String
use super::serial::get_serial;
type KEY = u8; //输入类型
lazy_static! { //缓冲区数据结构
  static ref INPUT_BUF: ArrayQueue<KEY> = ArrayQueue::new(128);
}
#[inline]
pub fn push_key(key: KEY) {
  //输入到缓冲区
  if INPUT_BUF.push(key).is_err() {
    warn!("Input buffer is full. Dropping key '{:?}'", key);
  }
}

#[inline]
pub fn try_pop_key() -> Option<KEY> {
  INPUT_BUF.pop()
}

//阻塞式从缓冲区取数据
pub fn pop_key() -> KEY {
  loop {
    if let Some(key) = try_pop_key() {
      return key;
    }
  }
}

const MAX_LINE_LENGTH: usize = 128;

pub fn get_line() -> String<MAX_LINE_LENGTH> {
  let mut line = String::<MAX_LINE_LENGTH>::new(); // 使用heapless创建一个固定大小的String
  while line.len() < MAX_LINE_LENGTH {
    let key = pop_key();
     // 假设pop_key()是阻塞调用，等待并返回下一个键入的字符
    match key {
      b'\r' => {
        println!(); // 打印换行符到屏幕或串口，结束输入
        break;
      }
      0x08 | 0x7f => {
        // Backspace or DEL
        if !line.is_empty() {
          line.pop(); // 从字符串中移除最后一个字符
          backspace(); // 执行屏幕上的退格操作
        }
      }
      _ => {
        if line.len() < MAX_LINE_LENGTH {
          print!("{}", key as char); // 回显
          line.push(key as char).unwrap_or_default();
        }
      }
    }
  }
  line
}

// 模拟退格操作的函数，发送0x08, 0x20, 0x08到串口以删除字符
fn backspace() {
  if let Some(mut serial_port) = get_serial() {
    serial_port.send(0x08);
    serial_port.send(0x20);
    serial_port.send(0x08);
  } else {
    info!("failed to get_serail");
  }
}
