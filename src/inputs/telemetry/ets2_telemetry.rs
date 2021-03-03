use std::{fmt, mem, ptr};

use std::ffi::c_void;

use crate::telemetry::{EventGame, Packet};
use crate::{Input, InputType};

pub enum ETS2Event {
    Paused,
    Started,
    Gameplay,
    FrameStart,
    FrameEnd,
    Configuration,
    NotValid,
}

impl ETS2Event {
    pub fn new(raw_event_type: u32) -> ETS2Event {
        match raw_event_type {
            1 => Self::FrameStart,
            2 => Self::FrameEnd,
            3 => Self::Paused,
            4 => Self::Started,
            5 => Self::Configuration,
            6 => Self::Gameplay,
            _ => Self::NotValid,
        }
    }

    pub fn to_eventgame(&self) -> EventGame {
        match self {
            Self::FrameStart => EventGame::FrameStartEvent,
            Self::FrameEnd => EventGame::FrameEndEvent,
            Self::Paused => EventGame::PausedEvent,
            Self::Started => EventGame::StartedEvent,
            Self::Configuration | Self::Gameplay => EventGame::OtherEvent,
            Self::NotValid => EventGame::NotValidEvent,
        }
    }
}

// use #[repr(C, packed)] not perform alignment. Data has been aligned manually
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
#[repr(C)]
struct euler {
    heading: f32,
    pitch: f32,
    roll: f32,
    // 12 bytes
}

// could implement with generics, however will run into unknown aligement issues
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
#[repr(C)]
struct fvector {
    x: f32,
    y: f32,
    z: f32,
    // 12 bytes
}
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
#[repr(C)]
struct dvector {
    x: f64,
    y: f64,
    z: f64,
    // 24 bytes
}
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
#[repr(C)]
struct fplacement {
    position: fvector,
    orientation: euler,
    // 24 bytes
}
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
#[repr(C)]
struct dplacement {
    position: dvector,
    orientation: euler,
    padding: u32,
    // 40 bytes
}
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
#[repr(C)]
pub struct frame_start {
    flags: u32,
    padding: u32,
    render_time: u64,
    simulation_time: u64,
    paused_simulation_time: u64,
    // 32 bytes
}
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
#[repr(C)]
pub struct frame_end {
    engine_gear: u32,
    speed: f32,
    engine_rpm: f32,
    input_steering: f32,
    input_throttle: f32,
    input_brake: f32,
    input_clutch: f32,
    effective_steering: f32,
    effective_throttle: f32,
    effective_brake: f32,
    effective_clutch: f32,
    cruise_control: f32,
    navigation_speed_limit: f32,
    padding: u32,

    cabin_angular_velocity: fvector,
    cabin_angular_acceleration: fvector,
    local_linear_velocity: fvector,
    local_angular_velocity: fvector,
    local_linear_acceleration: fvector,
    local_angular_acceleration: fvector,

    cabin_offset: fplacement,
    head_offset: fplacement,
    world_placement: dplacement,
    // 216 bytes
}

#[derive(Clone, Copy)]
#[repr(C)]
pub union event_data {
    pub frame_end_data: frame_end,
    pub frame_start_data: frame_start,
    pub no_data: u32,
}
#[derive(Clone, Copy)]
#[repr(C)]
pub struct telemetry_packet {
    pub type_: u32,
    pub length: u32,
    pub id: u64,
    pub time: u64,
    pub data: event_data,
}

impl telemetry_packet {
    pub fn new() -> telemetry_packet {
        let x: telemetry_packet = unsafe { mem::zeroed() };
        x
    }
}

impl fmt::Debug for telemetry_packet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("telemetry_packet")
            .field("type", &self.type_)
            .field("length", &self.length)
            .field("id", &self.id)
            .field("time", &self.time)
            .finish()
    }
}

impl Packet for telemetry_packet {
    fn parser(&mut self, address: *mut c_void) -> bool {
        #[repr(C)]
        struct Pair(bool, telemetry_packet);

        let rdata: Pair = unsafe { ptr::read(address as *const _) };
        *self = rdata.1; //telemetry_packet { ..packet };
        rdata.0
    }
}

impl Input for telemetry_packet {
    fn input_type(&self) -> InputType {
        InputType::Telemetry
    }

    fn event_type(&self) -> EventGame {
        ETS2Event::new(self.type_).to_eventgame()
    }

    fn header(&self) -> (u64, u32, u64, u32) {
        (self.id, self.type_, self.time, self.length)
    }
}
