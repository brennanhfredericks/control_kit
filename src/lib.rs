// specify telemetry format, hard coded to euro truck simulator 2

mod services;

pub use services::{Input, InputType, Process, ServiceError, ServiceType};

#[path = "inputs/synchronization.rs"]
pub mod synchronization;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
