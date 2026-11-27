//! The log output at the bottom of the window (the Python version uses a logging handler on a Text widget).

use chrono::Local;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Debug,
    Info,
    Warning,
    Error,
}

impl Level {
    fn name(self) -> &'static str {
        match self {
            Level::Debug => "DEBUG",
            Level::Info => "INFO",
            Level::Warning => "WARNING",
            Level::Error => "ERROR",
        }
    }
}

/// The messages shown in the log output. Debug messages are only kept while "Show debug messages" is on.
#[derive(Default)]
pub struct Log {
    lines: Vec<(Level, String)>,
    pub show_debug: bool,
    /// Set when a line was added, so the output scrolls to the end once.
    pub scroll_to_end: bool,
}

impl Log {
    pub fn add(&mut self, level: Level, message: impl AsRef<str>) {
        if level == Level::Debug && !self.show_debug {
            return;
        }
        // The format of Python's logging: '%(asctime)s - %(levelname)s - %(message)s'
        let time = Local::now().format("%Y-%m-%d %H:%M:%S,%3f");
        self.lines.push((level, format!("{time} - {} - {}", level.name(), message.as_ref())));
        self.scroll_to_end = true;
    }

    pub fn debug(&mut self, message: impl AsRef<str>) {
        self.add(Level::Debug, message);
    }

    pub fn info(&mut self, message: impl AsRef<str>) {
        self.add(Level::Info, message);
    }

    pub fn warning(&mut self, message: impl AsRef<str>) {
        self.add(Level::Warning, message);
    }

    pub fn error(&mut self, message: impl AsRef<str>) {
        self.add(Level::Error, message);
    }

    pub fn clear(&mut self) {
        self.lines.clear();
    }

    pub fn lines(&self) -> &[(Level, String)] {
        &self.lines
    }
}
