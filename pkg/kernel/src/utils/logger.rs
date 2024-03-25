use log::{Metadata, Record, Level};

pub fn init() {
    static LOGGER: Logger = Logger;
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(log::LevelFilter::Debug);
}

struct Logger;

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            match record.level() {
                Level::Error => println!("\x1b[31m[Error]{}\x1b[0m", record.args()), // Red
                Level::Warn => println!("\x1b[33m[Warn]{}\x1b[0m", record.args()),  // Yellow
                Level::Info => println!("\x1b[34m[Info]{}\x1b[0m", record.args()),  // Blue
                Level::Debug => println!("\x1b[32m[Debug]{}\x1b[0m", record.args()), // Green
                Level::Trace => println!("\x1b[35m[Trace]{}\x1b[0m", record.args()), // Magenta
            };
        }
    }

    fn flush(&self) {}
}
