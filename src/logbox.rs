use chrono::{DateTime, Local};
use std::sync::{LazyLock, RwLock};

/// the global logbox that stores the last log intended to be read by the user during runtime
pub static LOGBOX: LazyLock<RwLock<Logbox>> = LazyLock::new(|| RwLock::new(Logbox::default()));

pub struct Logbox {
    message: Option<String>,
    timestamp: DateTime<Local>,
}

impl Default for Logbox {
    fn default() -> Self {
        Self {
            message: None,
            timestamp: Local::now(),
        }
    }
}

impl Logbox {
    /// puts a new message into the logbox. the current time is captured automatically
    pub fn log(&mut self, message: &str) {
        self.message = Some(message.to_string());
        self.timestamp = Local::now();
    }

    /// returns the content of the logbox message in the format "message at timestamp"
    pub fn get_log_at_time(&self) -> String {
        if let Some(message) = self.message.clone() {
            message + " at " + &self.timestamp.format("%H:%M:%S").to_string()
        } else {
            "".to_string()
        }
    }
}
