//! The messages of the application; the bottom bar shows the last one.

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

/// The messages of the application. The window shows only the last one (in the bottom bar); debug messages are not
/// kept.
#[derive(Default)]
pub struct Log {
    lines: Vec<(Level, String)>,
    /// The last info, warning or error message and when it came.
    last: Option<(Level, String, std::time::Instant)>,
}

impl Log {
    pub fn add(&mut self, level: Level, message: impl AsRef<str>) {
        if level == Level::Debug {
            return;
        }
        // The format of Python's logging: '%(asctime)s - %(levelname)s - %(message)s'
        let time = Local::now().format("%Y-%m-%d %H:%M:%S,%3f");
        self.lines.push((level, format!("{time} - {} - {}", level.name(), message.as_ref())));
        self.last = Some((level, message.as_ref().to_string(), std::time::Instant::now()));
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
        self.last = None;
    }

    /// The last info, warning or error message, with how long ago it came.
    pub fn last(&self) -> Option<(Level, &str, std::time::Duration)> {
        self.last.as_ref().map(|(level, text, time)| (*level, text.as_str(), time.elapsed()))
    }

    pub fn lines(&self) -> &[(Level, String)] {
        &self.lines
    }
}
