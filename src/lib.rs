// #[cfg(windows)]
// extern crate winapi;
// extern crate wio;

mod services;
pub use services::{Input, InputType, Process, ServiceError, ServiceType, Services};

mod utils;

pub use utils::{str_to_wstring, windows_get_last_error};

#[path = "inputs/synchronization.rs"]
pub mod synchronization;

#[path = "inputs/telemetry.rs"]
pub mod telemetry;

#[path = "inputs/screencapture.rs"]
pub mod screencapture;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
