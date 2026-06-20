pub enum GuiMessage {
    Log(LogMessage),
    Progress(ProgressMessage),
}

pub struct LogMessage {
    pub message: String,
    pub level: LogLevel,
}

pub enum LogLevel {
    Info,
    Warning,
    Error,
}

pub struct ProgressMessage {
    pub progress: f32,
    pub progress_total: f32,
}
