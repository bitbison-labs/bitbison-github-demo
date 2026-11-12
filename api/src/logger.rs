//! The log facade's lines on standard error, one per record.

use log::{Level, LevelFilter, Log, Metadata, Record};

struct Stderr;

impl Log for Stderr {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            eprintln!("{:<5} {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

pub fn init() {
    // Set once, at startup: a second call is a no-op.
    if log::set_boxed_logger(Box::new(Stderr)).is_ok() {
        log::set_max_level(LevelFilter::Info);
    }
}
