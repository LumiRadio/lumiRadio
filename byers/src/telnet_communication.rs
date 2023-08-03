use tracing_unwrap::ResultExt;

pub trait ByersTelnet {
    fn request_song(&mut self, song: &str) -> Result<String, telnet::TelnetError>;
}

impl ByersTelnet for telnet::Telnet {
    fn request_song(&mut self, song: &str) -> Result<String, telnet::TelnetError> {
        self.write(&format!("srq.push {}\n", song).into_bytes())
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
}
