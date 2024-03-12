use log::{Level, Metadata, Record};

pub struct Logger;

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Debug
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!(
                "({}) {} ({}): {}",
                record.level(),
                get_time(),
                record.module_path().ok_or("unknown").unwrap(),
                record.args()
            );
        }
    }

    fn flush(&self) {}
}

fn get_time() -> chrono::DateTime<chrono::Local> {
    chrono::offset::Local::now()
}
