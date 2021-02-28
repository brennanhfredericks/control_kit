use std::ffi::c_void;
use std::{mem, ptr};

pub mod ETS2 {
    pub enum event_type {
        Paused = 3,
        Started = 4,
        Gameplay = 6,
        FrameStart = 1,
        FrameEnd = 2,
        Configuration = 5,
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

    #[repr(C)]
    pub union event_data {
        pub frame_end_data: frame_end,
        pub frame_start_data: frame_start,
        pub no_data: u32,
    }
    #[repr(C)]
    pub struct telemetry_packet {
        pub type_: u32,
        pub length: u32,
        pub id: u64,
        pub time: u64,
        pub data: event_data,
    }
}
