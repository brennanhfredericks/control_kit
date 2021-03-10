use crate::{Input, Process, ServiceError};
use std::sync::mpsc::Sender;

mod inputprocessmethod;

use inputprocessmethod::InputProcessMethod;

#[path = "telemetry/data_layout.rs"]
mod data_layout;

pub use data_layout::{DataPair, EventGame, Packet, PacketParser, SelectGame};

#[path = "telemetry/shared_memory.rs"]
mod shared_memory;

use shared_memory::SharedMemory;

pub struct Telemetry {
    telemetry_input: Box<dyn InputProcessMethod + Send>,
}

impl Telemetry {
    pub fn via_shared_memory(game: SelectGame) -> Telemetry {
        Telemetry {
            telemetry_input: Box::new(SharedMemory::new(game)),
        }
    }

    pub fn get_method(&self) -> &str {
        self.telemetry_input.method()
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
