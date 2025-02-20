use chrono::Local;
use std::sync::atomic::{AtomicU64, Ordering};

static TRANSACTION_COUNTER: AtomicU64 = AtomicU64::new(0);

const LOG_LEVEL: &str = "LOG";

pub struct Logger {
    prefix: String,
}

impl Logger {
    // Constructor function to create a new Logger instance
    pub fn new(prefix: String) -> Self {
        Self { prefix }
    }

    // Method to log a message with a prefix
    pub fn log(&self, message: String) {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        println!("{} {} {}", timestamp, self.prefix, message);
    }

    pub fn info(&self, message: String) {
        let log = format!("{} [INFO] {}", self.prefix_with_date(), message);
        println!("{}", log);
    }

    pub fn debug(&self, message: String) {
        let log = format!("{} [DEBUG] {}", self.prefix_with_date(), message);
        if LogLevel::new().is_debug() {
            println!("{}", log);
        }
    }

    pub fn error(&self, message: String) {
        let log = format!("{} [ERROR] {}", self.prefix_with_date(), message);
        println!("\x1b[31m{}\x1b[0m", log); // Red color for errors
    }

    pub fn success(&self, message: String) {
        let log = format!("{} [SUCCESS] {}", self.prefix_with_date(), message);
        println!("\x1b[32m{}\x1b[0m", log); // Green color for success
    }

    pub fn warning(&self, message: String) {
        let log = format!("{} [WARNING] {}", self.prefix_with_date(), message);
        println!("\x1b[33m{}\x1b[0m", log); // Yellow color for warnings
    }

    pub fn transaction(&self, message: String) {
        let count = TRANSACTION_COUNTER.fetch_add(1, Ordering::SeqCst);
        let log = format!("{} [TX:{}] {}", self.prefix_with_date(), count, message);
        println!("\x1b[36m{}\x1b[0m", log); // Cyan color for transactions
    }

    fn prefix_with_date(&self) -> String {
        let date = Local::now();
        format!(
            "[{}] {}",
            date.format("%Y-%m-%d %H:%M:%S%.3f"),
            self.prefix
        )
    }
}

struct LogLevel<'a> {
    level: &'a str,
}
impl LogLevel<'_> {
    fn new() -> Self {
        let level = LOG_LEVEL;
        LogLevel { level }
    }
    fn is_debug(&self) -> bool {
        self.level.to_lowercase().eq("debug")
    }
}
