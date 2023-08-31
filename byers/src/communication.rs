use std::path::PathBuf;

use tokio::io::AsyncReadExt;
use tracing::debug;
use tracing_unwrap::ResultExt;

#[async_trait::async_trait]
pub trait LiquidsoapCommunication {
    type Error;
    async fn send(&mut self, command: &str) -> Result<(), Self::Error>;
    async fn send_wait(&mut self, command: &str) -> Result<String, Self::Error>;

    async fn request_song(&mut self, song: &str) -> Result<String, Self::Error> {
        self.send_wait(&format!("srq.push {}", song)).await
    }
    async fn priority_request(&mut self, song: &str) -> Result<String, Self::Error> {
        self.send_wait(&format!("prioq.push {}", song)).await
    }
}

#[async_trait::async_trait]
impl LiquidsoapCommunication for telnet::Telnet {
    type Error = telnet::TelnetError;

    async fn send_wait(&mut self, command: &str) -> Result<String, Self::Error> {
        self.write(command.as_bytes())
            .expect_or_log("Failed to write to telnet");

        let result = loop {
            let event = self.read().expect_or_log("Failed to read from telnet");

            if let telnet::Event::Data(data) = event {
                break std::str::from_utf8(&data)
                    .map(ToString::to_string)
                    .expect_or_log("Failed to parse utf8");
            }
        };

        Ok(result)
    }

    async fn send(&mut self, command: &str) -> Result<(), Self::Error> {
        self.write(command.as_bytes())
            .expect_or_log("Failed to write to telnet");

        Ok(())
    }
}

pub struct ByersUnixStream {
    stream: tokio::net::UnixStream,
}

impl ByersUnixStream {
    pub async fn new() -> Result<Self, std::io::Error> {
        // wait until /usr/src/app/ls/lumiradio.sock exists
        let stream = loop {
            if PathBuf::from("/usr/src/app/ls/lumiradio.sock").exists() {
                let stream_result = tokio::net::UnixStream::connect(PathBuf::from(
                    "/usr/src/app/ls/lumiradio.sock",
                )).await;
                if let Ok(stream) = stream_result {
                    break stream;
                }
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        };

        Ok(Self { stream })
    }

    pub async fn read_until_end(&mut self) -> Result<String, std::io::Error> {
        let mut buf = Vec::new();
        let mut read_buffer = [0; 4096];

        loop {
            self.stream.readable().await?;
            let bytes_read = match self.stream.try_read(&mut read_buffer) {
                Ok(n) => {
                    debug!("Read {} bytes from liquidsoap", n);
                    n
                },
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    debug!("Would block, reading from liquidsoap");
                    continue;
                },
                Err(e) => {
                    return Err(e);
                }
            };
            buf.extend_from_slice(&read_buffer[..bytes_read]);

            if let Some(end_idx) = buf.windows(3).position(|window| window == b"END") {
                return Ok(String::from_utf8_lossy(&buf[..end_idx]).to_string());
            }
        }
    }

    pub async fn write(&mut self, data: &[u8]) -> Result<(), std::io::Error> {
        loop {
            self.stream.writable().await?;
            match self.stream.try_write(data) {
                Ok(n) => {
                    debug!("Wrote {} bytes to liquidsoap", n);
                    break;
                },
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    debug!("Would block, writing to liquidsoap");
                },
                Err(e) => {
                    return Err(e);
                }
            }
        }

        Ok(())
    }

    pub async fn write_str(&mut self, data: &str) -> Result<(), std::io::Error> {
        debug!("Writing to liquidsoap: {}", data);
        self.write(data.as_bytes()).await
    }

    pub async fn write_line(&mut self, data: &str) -> Result<(), std::io::Error> {
        let data_with_newline = format!("{}\n", data);
        self.write_str(&data_with_newline).await?;

        Ok(())
    }

    pub async fn write_str_and_wait_for_response(
        &mut self,
        data: &str,
    ) -> Result<String, std::io::Error> {
        let data_with_newline = format!("{}\n", data);
        self.write_str(&data_with_newline).await?;
        
        let result = self.read_until_end().await?;
        Ok(result)
    }
}

#[async_trait::async_trait]
impl LiquidsoapCommunication for ByersUnixStream {
    type Error = std::io::Error;

    async fn send_wait(&mut self, command: &str) -> Result<String, Self::Error> {
        self.write_str_and_wait_for_response(command).await
    }

    async fn send(&mut self, command: &str) -> Result<(), Self::Error> {
        self.write_line(command).await
    }
}
