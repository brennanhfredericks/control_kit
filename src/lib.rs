// specify telemetry format, hard coded to euro truck simulator 2
pub use ets2_telemetry::ATS_ETS2Telemetry::telemetry_packet;
mod services;

pub use services::{Input, InputType, Process, ServiceError, ServiceType};

mod synchronization;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
