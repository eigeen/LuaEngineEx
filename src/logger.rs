use log::{Metadata, Record};

pub struct MHWLogger {
    prefix: String,
}

impl MHWLogger {
    pub fn new() -> Self {
        Self {
            prefix: "LuaEngineEx".to_string(),
        }
    }
}

impl log::Log for MHWLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::Level::Debug
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            mhw_toolkit::logger::log_to_loader(
                record.level().into(),
                &format!("[{}] {} - {}", self.prefix, record.level(), record.args()),
            );
        }
    }

    fn flush(&self) {}
}
