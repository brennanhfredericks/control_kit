use std::ffi::c_void;

use crate::Input;

mod ets2_telemetry;

#[derive(Debug, Clone, Copy)]
pub enum SelectGame {
    ETS2,
}

impl SelectGame {
    pub fn get_events(&self) {}
}

#[derive(Debug, PartialEq, Eq)]
pub enum EventGame {
    PausedEvent,
    StartedEvent,
    FrameEndEvent,
    FrameStartEvent,
    OtherEvent,    // valid other game events
    NotValidEvent, // not a valid events possible data corruptions
}

pub struct DataPair(pub bool, pub Box<dyn Input + Send>);
pub trait Packet: Input {
    fn parser(&mut self, address: *mut c_void) -> bool;
}
#[derive(Debug, Clone, Copy)]
pub struct PacketParser {
    game: SelectGame,
}

impl PacketParser {
    pub fn new(selected_game: SelectGame) -> PacketParser {
        PacketParser {
            game: selected_game,
        }
    }

    pub fn data(self, address: *mut c_void) -> DataPair {
        // could use a match statement for enum type i.e. game telemetry data format

        let mut p = match self.game {
            SelectGame::ETS2 => ets2_telemetry::telemetry_packet::new(),
        };

        let is_alive = p.parser(address);

        DataPair {
            0: is_alive,
            1: Box::new(p),
        }
    }
}
