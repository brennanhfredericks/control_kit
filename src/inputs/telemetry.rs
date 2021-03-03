use crate::{Input, Process, ServiceError};
use std::sync::mpsc::Sender;

#[path = "telemetry/data_layout.rs"]
mod data_layout;

pub use data_layout::{DataPair, EventGame, Packet, PacketParser, SelectGame};

#[path = "telemetry/shared_memory.rs"]
mod shared_memory;

use shared_memory::SharedMemory;

// wrapper trait for Input, it is expected that Telemetry can be retrieved via
// memory-mapped file, pipe or socket

pub trait TelemetryInputMethod {
    fn start(&mut self) -> Result<(), ServiceError>;
    fn stop(&mut self);
    fn join(&mut self);
    fn retrieval_method(&self) -> &str;
    fn set_transmitter(&mut self, transmitter: Sender<Box<dyn Input + Send>>);
}

pub struct Telemetry {
    telemetry_input: Box<dyn TelemetryInputMethod + Send>,
}

impl Telemetry {
    pub fn via_shared_memory(game: SelectGame) -> Telemetry {
        Telemetry {
            telemetry_input: Box::new(SharedMemory::new(game)),
        }
    }

    pub fn set_transmitter(&mut self, transmitter: Sender<Box<dyn Input + Send>>) {
        self.telemetry_input.set_transmitter(transmitter);
    }
}

impl Process for Telemetry {
    fn start(&mut self) -> Result<(), ServiceError> {
        self.telemetry_input.start()?;
        Ok(())
    }

    fn stop(&mut self) -> Result<(), ServiceError> {
        self.telemetry_input.stop();
        Ok(())
    }

    fn join(&mut self) {
        self.telemetry_input.join();
    }
}
