pub enum GuiMessage {
    Log(LogMessage),
    Progress(ProgressMessage),
    Finished,
}

pub struct LogMessage {
    pub message: String,
    pub level: LogLevel,
}

pub enum LogLevel {
    Info,
    Warning,
    Error,
    Good,
}

pub struct ProgressMessage {
    pub progress: f32,
    pub progress_total: f32,
}
