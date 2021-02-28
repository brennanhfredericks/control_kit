use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};

use std::thread;

use crate::{Input, InputType, Process, ServiceError};

// Responsible for aligning data in a sensible manner.
// i.e when telemetry indicates pause state all other inputs should be discared or stopped
struct Synchronization {
    transmitter: Sender<Box<dyn Input + Send>>,
    receiver: Option<Receiver<Box<dyn Input + Send>>>,
    sentinal: Arc<Mutex<bool>>,
    handle: Option<thread::JoinHandle<()>>,
}

impl Synchronization {
    pub fn new() -> Synchronization {
        let (transmitter, receiver) = channel();
        let receiver = Some(receiver);
        Synchronization {
            transmitter,
            receiver,
            sentinal: Arc::new(Mutex::new(false)),
            handle: None,
        }
    }

    //return cloned transmitter
    pub fn get_transmitter(&self) -> Sender<Box<dyn Input + Send>> {
        self.transmitter.clone()
    }
}

impl Process for Synchronization {
    fn start(&mut self) -> Result<(), ServiceError> {
        if self.receiver.is_none() {
            return Err(ServiceError::already_active);
        }

        let receiver = self.receiver.take().unwrap();
        let sentinal = Arc::new(Mutex::new(true));

        self.sentinal = sentinal.clone();

        let mut input_buf: Vec<InputType> = Vec::new();

        let process = move || loop {
            if !*sentinal.lock().unwrap() {
                println!("exiting syncronization loop");
                break;
            }

            for input in receiver.try_iter() {
                // if packet is none loop will exit
                println!("input type recieved: {:?}", input.input_type());
            }
        };

        let process = thread::spawn(process);
        self.handle = Some(process);
        Ok(())
    }

    fn stop(&mut self) -> Result<(), ServiceError> {
        if self.receiver.is_some() {
            return Err(ServiceError::not_active);
        }
        {
            *self.sentinal.lock().unwrap() = false;
        }
        Ok(())
    }

    fn join(mut self) {
        if self.handle.is_some() {
            self.handle.unwrap().join().unwrap();
            self.handle = None;
        }
    }
}
