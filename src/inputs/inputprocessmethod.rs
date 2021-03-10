use crate::{Input, ServiceError};
use std::sync::mpsc::Sender;

pub trait InputProcessMethod {
    fn start(&mut self) -> Result<(), ServiceError>;
    fn stop(&mut self);
    fn join(&mut self);
    fn method(&self) -> &str;
    fn set_transmitter(&mut self, transmitter: Sender<Box<dyn Input + Send>>);
}
