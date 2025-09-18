use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use tokio::sync::mpsc::UnboundedSender;

const START_TAG: &str = "<FORGE_EVENT";
const END_TAG: &str = "</FORGE_EVENT>";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentEvent {
    pub agent_id: String,
    pub event_name: Option<String>,
    pub payload: Value,
    pub raw: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EventParseError {
    NonUtf8,
    MissingJson { raw: String },
    InvalidJson { raw: String, message: String },
}

#[derive(Debug, Clone, PartialEq)]
struct ParsedEvent {
    event_name: Option<String>,
    payload: Value,
    raw: String,
}

struct PtyEventParser {
    buffer: Vec<u8>,
}

impl PtyEventParser {
    fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    fn feed(&mut self, chunk: &[u8]) -> Vec<Result<ParsedEvent, EventParseError>> {
        self.buffer.extend_from_slice(chunk);
        let mut results = Vec::new();
        loop {
            let start = Self::find_tag(&self.buffer, START_TAG.as_bytes());
            let Some(start_idx) = start else {
                self.trim_buffer();
                break;
            };
            if start_idx > 0 {
                self.buffer.drain(..start_idx);
            }
            let end_rel = Self::find_tag(&self.buffer, END_TAG.as_bytes());
            let Some(end_idx) = end_rel else {
                break;
            };
            let event_end = end_idx + END_TAG.len();
            let bytes: Vec<u8> = self.buffer.drain(..event_end).collect();
            let raw = match String::from_utf8(bytes) {
                Ok(value) => value,
                Err(_) => {
                    results.push(Err(EventParseError::NonUtf8));
                    continue;
                }
            };
            match Self::parse_raw(&raw) {
                Ok(parsed) => results.push(Ok(parsed)),
                Err(err) => results.push(Err(err)),
            }
        }
        results
    }

    fn parse_raw(raw: &str) -> Result<ParsedEvent, EventParseError> {
        let Some(tag_end) = raw.find('>') else {
            return Err(EventParseError::MissingJson {
                raw: raw.to_string(),
            });
        };
        let start_tag = &raw[..tag_end];
        if !raw.ends_with(END_TAG) {
            return Err(EventParseError::MissingJson {
                raw: raw.to_string(),
            });
        }
        let payload_str = raw[tag_end + 1..raw.len() - END_TAG.len()].trim();
        if payload_str.is_empty() {
            return Err(EventParseError::MissingJson {
                raw: raw.to_string(),
            });
        }
        let payload = serde_json::from_str::<Value>(payload_str).map_err(|err| {
            EventParseError::InvalidJson {
                raw: raw.to_string(),
                message: err.to_string(),
            }
        })?;
        let name = Self::extract_name(start_tag);
        Ok(ParsedEvent {
            event_name: name,
            payload,
            raw: raw.to_string(),
        })
    }

    fn extract_name(tag: &str) -> Option<String> {
        tag.split_whitespace().skip(1).find_map(|part| {
            part.strip_prefix("name=\"")
                .and_then(|value| value.strip_suffix('\"'))
                .map(|value| value.to_string())
        })
    }

    fn find_tag(buffer: &[u8], marker: &[u8]) -> Option<usize> {
        buffer
            .windows(marker.len())
            .position(|window| window == marker)
    }

    fn trim_buffer(&mut self) {
        if self.buffer.len() > START_TAG.len() {
            let drain_until = self.buffer.len() - START_TAG.len();
            self.buffer.drain(..drain_until);
        }
    }
}

#[derive(Clone)]
pub struct AgentEventSender {
    sender: UnboundedSender<AgentEvent>,
}

impl AgentEventSender {
    pub fn new(sender: UnboundedSender<AgentEvent>) -> Self {
        Self { sender }
    }

    pub fn sender(&self) -> UnboundedSender<AgentEvent> {
        self.sender.clone()
    }
}

pub struct AgentProcess {
    pub id: String,
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
}

impl AgentProcess {
    pub fn spawn(id: &str, command: Vec<&str>, events: UnboundedSender<AgentEvent>) -> Self {
        let pty_system = NativePtySystem::default();
        let pair = pty_system
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .unwrap();

        let mut cmd = CommandBuilder::new(command[0]);
        cmd.args(&command[1..]);

        let mut _child = pair.slave.spawn_command(cmd).unwrap();
        let mut reader = pair.master.try_clone_reader().unwrap();
        let writer = Arc::new(Mutex::new(
            pair.master.take_writer().unwrap() as Box<dyn Write + Send>
        ));

        let agent_id = id.to_string();
        let event_sender = events.clone();
        thread::spawn(move || {
            let mut buffer = [0u8; 1024];
            let mut parser = PtyEventParser::new();
            'read: loop {
                match reader.read(&mut buffer) {
                    Ok(len) => {
                        if len == 0 {
                            break;
                        }
                        let chunk = &buffer[..len];
                        let output = String::from_utf8_lossy(chunk);
                        println!("[Agent {}]: {}", agent_id, output);
                        for result in parser.feed(chunk) {
                            match result {
                                Ok(parsed) => {
                                    let event = AgentEvent {
                                        agent_id: agent_id.clone(),
                                        event_name: parsed.event_name,
                                        payload: parsed.payload,
                                        raw: parsed.raw,
                                    };
                                    if event_sender.send(event).is_err() {
                                        break 'read;
                                    }
                                }
                                Err(err) => {
                                    println!("[Agent {} parse error]: {:?}", agent_id, err);
                                }
                            }
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        Self {
            id: id.to_string(),
            writer,
        }
    }

    pub fn send_command(&self, command: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.write_line(command)
    }

    pub fn send_structured_event(
        &self,
        name: &str,
        payload: &Value,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let formatted = Self::format_structured_event(name, payload);
        self.write_line(&formatted)
    }

    pub fn format_structured_event(name: &str, payload: &Value) -> String {
        let payload_text = payload.to_string();
        format!("<FORGE_EVENT name=\"{name}\">{payload_text}</FORGE_EVENT>")
    }

    fn write_line(&self, line: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut writer = self.writer.lock().unwrap();
        writer.write_all(line.as_bytes())?;
        writer.write_all(b"\n")?;
        writer.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parser_handles_split_tags() {
        let mut parser = PtyEventParser::new();
        assert!(parser.feed(b"normal <FORGE_EVENT name=\"PLAN").is_empty());
        let results = parser.feed(b"_COMPLETE\">{\"status\":\"done\"}</FORGE_EVENT> tail");
        assert_eq!(results.len(), 1);
        let event = results[0].as_ref().unwrap();
        assert_eq!(event.event_name.as_deref(), Some("PLAN_COMPLETE"));
        assert_eq!(event.payload["status"], "done");
    }

    #[test]
    fn parser_flags_invalid_json() {
        let mut parser = PtyEventParser::new();
        let results = parser.feed(b"<FORGE_EVENT name=\"BROKEN\">not json</FORGE_EVENT>");
        assert_eq!(results.len(), 1);
        assert!(matches!(
            results[0],
            Err(EventParseError::InvalidJson { .. })
        ));
    }

    #[test]
    fn structured_events_are_formatted_with_forge_tag() {
        let payload = json!({"foo": "bar"});
        let formatted = AgentProcess::format_structured_event("TEST", &payload);
        assert_eq!(
            formatted,
            "<FORGE_EVENT name=\"TEST\">{\"foo\":\"bar\"}</FORGE_EVENT>"
        );
    }
}
