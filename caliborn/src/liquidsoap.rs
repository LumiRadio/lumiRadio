use std::{path::{Path, PathBuf}, time::Duration};

use tokio::{io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter}, net::{unix::{OwnedReadHalf, OwnedWriteHalf}, UnixStream}, time::timeout};
use tracing::warn;

const RECONNECT_DELAY: Duration = Duration::from_millis(100);
const READ_TIMEOUT: Duration = Duration::from_secs(5);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(3);
const BUF_SIZE: usize = 8192;
const END_MARKER: &[u8] = b"END";

#[derive(thiserror::Error, Debug)]
pub enum LiquidsoapError {
    #[error("The requested socket does not exist: {0}")]
    SocketDoesNotExist(String),

    #[error("Connection timeout")]
    ConnectionTimeout,

    #[error("Read timeout")]
    ReadTimeout,

    #[error("Invalid UTF-8 in response: {0}")]
    InvalidUtf8(#[from] std::string::FromUtf8Error),

    #[error("IO error: {0}")]
    Io(std::io::Error),
}

pub struct LiquidsoapClient {
    path: PathBuf,
    reader: BufReader<OwnedReadHalf>,
    writer: BufWriter<OwnedWriteHalf>,
}

impl LiquidsoapClient {
    pub async fn new<P: AsRef<Path>>(path: P) -> Result<Self, LiquidsoapError> {
        let path = path.as_ref().to_path_buf();
        let stream = Self::connect_with_retry(&path).await?;

        let (read_half, write_half) = stream.into_split();
        let reader = BufReader::with_capacity(BUF_SIZE, read_half);
        let writer = BufWriter::with_capacity(BUF_SIZE, write_half);

        Ok(Self { path, reader, writer })
    }

    async fn connect_with_retry(path: &Path) -> Result<UnixStream, LiquidsoapError> {
        let mut attempts = 0;
        let max_attempts = 30; // 3 seconds with 100ms delay
        
        loop {
            attempts += 1;
            
            if path.exists() {
                match timeout(CONNECT_TIMEOUT, UnixStream::connect(path)).await {
                    Ok(Ok(stream)) => {
                        tracing::debug!("Successfully connected to Liquidsoap socket at {}", path.display());
                        return Ok(stream);
                    }
                    Ok(Err(e)) => {
                        tracing::warn!("Failed to connect to Liquidsoap socket: {}", e);
                    }
                    Err(_) => {
                        tracing::warn!("Connection attempt timed out");
                    }
                }
            } else if attempts >= max_attempts {
                return Err(LiquidsoapError::SocketDoesNotExist(path.display().to_string()));
            }
            
            tracing::debug!("Waiting for Liquidsoap socket to appear at {}", path.display());
            tokio::time::sleep(RECONNECT_DELAY).await;
        }
    }

    pub async fn reconnect(&mut self) -> Result<(), LiquidsoapError> {
        tracing::debug!("Reconnecting to Liquidsoap socket at {}", self.path.display());
        let stream = Self::connect_with_retry(&self.path).await?;

        let (read_half, write_half) = stream.into_split();
        self.reader = BufReader::with_capacity(BUF_SIZE, read_half);
        self.writer = BufWriter::with_capacity(BUF_SIZE, write_half);

        Ok(())
    }

    pub async fn read_until_end(&mut self) -> Result<String, LiquidsoapError> {
        let mut buffer = Vec::with_capacity(BUF_SIZE);
        let mut chunk = vec![0; 1024];

        loop {
            match timeout(READ_TIMEOUT, self.reader.read(&mut chunk)).await {
                Ok(Ok(0)) => {
                    tracing::error!("Liquidsoap socket closed unexpectedly");
                    return Err(LiquidsoapError::Io(std::io::Error::new(
                        std::io::ErrorKind::UnexpectedEof,
                        "Socket closed unexpectedly",
                    )));
                }
                Ok(Ok(n)) => {
                    buffer.extend_from_slice(&chunk[..n]);

                    if buffer.windows(END_MARKER.len()).any(|window| window == END_MARKER) {
                        if let Some(end_idx) = buffer.windows(END_MARKER.len())
                            .position(|window| window == END_MARKER) {
                                let response = String::from_utf8(buffer[..end_idx].to_vec())?;
                                return Ok(response);
                            }
                    }
                }
                Ok(Err(e)) => {
                    return Err(LiquidsoapError::Io(e));
                },
                Err(_) => {
                    return Err(LiquidsoapError::ReadTimeout);
                }
            }
        }
    }

    pub async fn write(&mut self, data: &[u8]) -> Result<(), LiquidsoapError> {
        self.writer.write_all(data).await.map_err(LiquidsoapError::Io)?;
        self.writer.flush().await.map_err(LiquidsoapError::Io)?;
        Ok(())
    }

    pub async fn write_str(&mut self, data: &str) -> Result<(), LiquidsoapError> {
        self.write(data.as_bytes()).await
    }

    pub async fn write_line(&mut self, data: &str) -> Result<(), LiquidsoapError> {
        self.write_str(&format!("{}\n", data)).await
    }

    pub async fn command(&mut self, cmd: &str) -> Result<String, LiquidsoapError> {
        self.write_line(cmd).await?;
        self.read_until_end().await
    }

    pub async fn command_with_reconnect(&mut self, cmd: &str) -> Result<String, LiquidsoapError> {
        match self.command(cmd).await {
            Ok(response) => Ok(response),
            Err(e) => {
                warn!("Failed to execute command: {}", e);
                self.reconnect().await?;
                self.command(cmd).await
            }
        }
    }

    pub async fn shutdown(mut self) -> Result<(), LiquidsoapError> {
        self.writer.flush().await.map_err(LiquidsoapError::Io)?;
        
        let write_half = self.writer.get_mut();
        let _read_half = self.reader.get_mut();

        write_half.shutdown().await.map_err(LiquidsoapError::Io)?;
        
        Ok(())
    }
}
