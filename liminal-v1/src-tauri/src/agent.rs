use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;

pub struct AgentProcess {
    pub id: String,
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
}

impl AgentProcess {
    pub fn spawn(id: &str, command: Vec<&str>) -> Self {
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
        thread::spawn(move || {
            let mut buffer = [0u8; 1024];
            loop {
                match reader.read(&mut buffer) {
                    Ok(len) => {
                        if len == 0 {
                            break;
                        }
                        let output = String::from_utf8_lossy(&buffer[..len]);
                        println!("[Agent {}]: {}", agent_id, output);
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
        let mut writer = self.writer.lock().unwrap();
        writer.write_all(command.as_bytes())?;
        writer.write_all(b"\n")?;
        writer.flush()?;
        Ok(())
    }
}
