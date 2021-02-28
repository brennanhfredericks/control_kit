// specify telemetry format, hard coded to euro truck simulator 2

mod services;
pub use services::{Input, InputType, Process, ServiceError, ServiceType};

mod utils;

pub use utils::{str_to_wstring, windows_get_last_error};

#[path = "inputs/synchronization.rs"]
pub mod synchronization;

#[path = "inputs/telemetry.rs"]
pub mod telemetry;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
