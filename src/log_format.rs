use std::time::SystemTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stream {
    Stdout,
    Stderr,
}

impl Stream {
    fn as_str(&self) -> &'static str {
        match self {
            Stream::Stdout => "stdout",
            Stream::Stderr => "stderr",
        }
    }
}

#[derive(Debug, Clone)]
pub struct LogLine {
    pub timestamp: SystemTime,
    pub stream: Stream,
    pub content: String,
}

pub fn format_line(line: &LogLine) -> String {
    let ts = humantime::format_rfc3339_seconds(line.timestamp);
    format!("[{}][{}] {}", ts, line.stream.as_str(), line.content)
}

pub fn parse_line(raw: &str) -> Option<LogLine> {
    let rest = raw.strip_prefix('[')?;
    let (ts_str, rest) = rest.split_once(']')?;
    let rest = rest.strip_prefix('[')?;
    let (stream_str, rest) = rest.split_once(']')?;
    let content = rest.strip_prefix(' ').unwrap_or(rest);

    let timestamp = humantime::parse_rfc3339(ts_str).ok()?;
    let stream = match stream_str {
        "stdout" => Stream::Stdout,
        "stderr" => Stream::Stderr,
        _ => return None,
    };

    Some(LogLine {
        timestamp,
        stream,
        content: content.to_string(),
    })
}
