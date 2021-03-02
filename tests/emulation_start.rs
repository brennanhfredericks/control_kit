use std::io::{BufRead, BufReader, Error, ErrorKind};
use std::process::{ChildStdout, Command, Stdio};

pub struct ets2_emulation;

impl ets2_emulation {
    pub fn start_with_stdout() -> BufReader<ChildStdout> {
        let stdout = Command::new(".\\tests.\\TelemetryEmulation.exe")
            .stdout(Stdio::piped())
            .spawn()
            .unwrap()
            .stdout
            .unwrap();

        BufReader::new(stdout)
    }
}
