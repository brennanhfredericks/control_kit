use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::telemetry::EventGame;
use crate::{Input, InputType, Process, ServiceError};

// Responsible for aligning data in a sensible manner.
// i.e when telemetry indicates pause state all other inputs should be discared or stopped
pub struct Synchronization {
    input_transmitter: Sender<Box<dyn Input + Send>>,
    input_receiver: Option<Receiver<Box<dyn Input + Send>>>,
    output_transmitter: Option<Sender<Vec<Box<dyn Input + Send>>>>,
    sentinal: Arc<Mutex<bool>>,
    handle: Option<thread::JoinHandle<()>>,
}

impl Synchronization {
    pub fn new() -> Synchronization {
        let (input_transmitter, input_receiver) = channel();
        let input_receiver = Some(input_receiver);
        Synchronization {
            input_transmitter,
            input_receiver,
            output_transmitter: None,
            sentinal: Arc::new(Mutex::new(false)),
            handle: None,
        }
    }

    //return cloned transmitter
    pub fn get_input_transmitter(&self) -> Sender<Box<dyn Input + Send>> {
        self.input_transmitter.clone()
    }

    pub fn set_output_transmitter(&mut self, transmitter: Sender<Vec<Box<dyn Input + Send>>>) {
        self.output_transmitter = Some(transmitter);
    }
}

impl Process for Synchronization {
    fn start(&mut self) -> Result<(), ServiceError> {
        if self.input_receiver.is_none() {
            return Err(ServiceError::AlreadyActive);
        }

        let receiver = self.input_receiver.take().unwrap();
        let transmitter = self.output_transmitter.take().unwrap();

        let sentinal = Arc::new(Mutex::new(true));

        self.sentinal = sentinal.clone();

        let mut input_buf: Vec<Box<dyn Input + Send>> = Vec::new();

        // push on to input_buf from StartFrameEvent .... until EndFrameEvent received
        // drain and collect input_buf into new vec and pass onto distribution services
        //

        let process = move || {
            let mut in_game_driving: bool = false;

            loop {
                if !*sentinal.lock().unwrap() {
                    println!("exiting syncronization loop");
                    break;
                }

                for input in receiver.try_iter() {
                    // if packet is none loop will
                    match input.input_type() {
                        InputType::Telemetry => {
                            match input.event_type() {
                                EventGame::FrameStartEvent => {
                                    if in_game_driving {
                                        input_buf.push(input);
                                    }
                                }
                                EventGame::FrameEndEvent => {
                                    if in_game_driving {
                                        input_buf.push(input);
                                        // drain vec into new struct

                                        let groupify: Vec<Box<dyn Input + Send>> =
                                            input_buf.drain(..).collect();
                                        transmitter.send(groupify).unwrap();
                                    }
                                }
                                EventGame::PausedEvent => {
                                    //stop pushing data on the distribution services
                                    println!("received PausedEvent");
                                    in_game_driving = false;
                                }
                                EventGame::StartedEvent => {
                                    //start pushing data on the distribution services again
                                    println!("received StartedEvent");
                                    in_game_driving = true;
                                }
                                EventGame::OtherEvent => {
                                    if in_game_driving {
                                        input_buf.push(input);
                                    }
                                }
                                EventGame::NotValidEvent => {
                                    println!(
                                        "received NotValidEvent: could be corruption or #[repr(c)]"
                                    )
                                }
                            }
                            //println!(" EventGame: {:?}", input.event_type())
                        }
                        InputType::User => {
                            if in_game_driving {
                                input_buf.push(input);
                            }
                        }
                        InputType::Image => {
                            if in_game_driving {
                                input_buf.push(input);
                            }
                        }
                    };
                }
            }
        };

        let process = thread::spawn(process);
        self.handle = Some(process);
        Ok(())
    }

    fn stop(&mut self) -> Result<(), ServiceError> {
        if self.input_receiver.is_some() {
            return Err(ServiceError::NotActive);
        }
        {
            *self.sentinal.lock().unwrap() = false;
        }
        Ok(())
    }

    fn join(&mut self) {
        if self.handle.is_some() {
            self.handle.take().unwrap().join().unwrap();
        }
    }
}
