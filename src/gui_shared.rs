//! Shares variables needed for sender-reciever communication with gui.

/// Types of messages sent to gui by other modules
pub enum GuiMessage {
    /// Struct of message displayed in log view widget
    Log(LogMessage),
    /// Struct of values of the progressbars
    Progress(ProgressMessage),
    /// Bool to gray out Start button during scanning
    Finished,
}

/// Struct of text-message and it's level
pub struct LogMessage {
    pub message: String,
    pub level: LogLevel,
}

/// Enum of log levels for messages that are being parsed in main and colorized accordingly
pub enum LogLevel {
    Info,
    Warning,
    Error,
    Good,
}

/// Values of progressbars
pub struct ProgressMessage {
    pub progress: f32,
    pub progress_total: f32,
}

/// Struct of the app options. It is set by the options panel in egui and then passed to `scanner.rs`
#[derive(Copy, Clone)]
pub struct Options {
    pub scan_video: bool,
    pub scan_assets: bool,
    pub scan_history: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            scan_video: true,
            scan_assets: true,
            scan_history: true,
        }
    }
}
