use std::io;

use crate::{
    log::{LogEntry, Logger},
    Serialize,
};

pub struct BufferLogger {
    entries: Vec<LogEntry>,
}

impl BufferLogger {
    #[inline]
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        for entry in &self.entries {
            entry.compute_size();
            entry
                .serialize_nested_with_cached_size(None, &mut bytes)
                .unwrap();
        }

        bytes
    }

    #[inline]
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    #[inline]
    pub fn pluck(&mut self) -> Vec<u8> {
        let bytes = self.bytes();
        self.clear();
        bytes
    }
}

impl Logger for BufferLogger {
    #[inline]
    fn log(&mut self, entry: LogEntry) -> io::Result<()> {
        self.entries.push(entry);
        Ok(())
    }
}
