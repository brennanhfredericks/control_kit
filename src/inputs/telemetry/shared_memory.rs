use bindings::{
    windows::win32::system_services::{
        MapViewOfFile, OpenEventW, OpenFileMappingW, OpenMutexW, ReleaseMutex, ResetEvent,
        SetEvent, UnmapViewOfFile, WaitForMultipleObjects, HANDLE,
    },
    windows::win32::windows_programming::CloseHandle,
    windows::BOOL,
};

use crate::{str_to_wstring, windows_get_last_error, Input, ServiceError};
use std::ffi::c_void;

use std::result::Result;

use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::telemetry::{DataPair, PacketParser, SelectGame, TelemetryInputMethod};
//use to setup windows inter process communication and sychronization objects
struct InterProcessCommunication {
    hmapping_obj: Option<HANDLE>,
    memory_file_start_address: Option<*mut c_void>,
    hmutex_obj: Option<HANDLE>,
    hread_event_obj: Option<HANDLE>,
    hwrite_event_obj: Option<HANDLE>,
}

impl InterProcessCommunication {
    pub fn new() -> InterProcessCommunication {
        InterProcessCommunication {
            hmapping_obj: None,
            memory_file_start_address: None,
            hmutex_obj: None,
            hread_event_obj: None,
            hwrite_event_obj: None,
        }
    }

    // the named objects is hardcode, should change in future
    pub fn connect(&mut self) -> Result<(), ServiceError> {
        // open windows named file mapping object in read only mode
        let hmapping_obj_name = str_to_wstring("ETS2Telemetry");
        let hmapping_obj =
            unsafe { OpenFileMappingW(0x0004, BOOL::from(false), hmapping_obj_name.as_ptr()) };

        // check if operation was succesfull
        windows_get_last_error("OpenFileMapping")?;

        // set handle
        self.hmapping_obj = Some(hmapping_obj); // clone occurs

        // map view of file using file mapping obj, last three paramaters zero to map the entire memory_file
        let memory_file_start_address = unsafe { MapViewOfFile(hmapping_obj, 0x004, 0, 0, 0) };

        // check if operation was succefull
        windows_get_last_error("MapViewOfFile")?;

        self.memory_file_start_address = Some(memory_file_start_address);

        //open windows named mutex object with permission to access_modify
        let hmutex_obj_name = str_to_wstring("ETS2TelemetryMutex");
        let hmutex_obj =
            unsafe { OpenMutexW(0x00100000, BOOL::from(false), hmutex_obj_name.as_ptr()) };

        // check if operation is successfull
        windows_get_last_error("OpenMutexW")?;

        self.hmutex_obj = Some(hmutex_obj);

        // open windows name event object with permission to synchronize and modify,
        // client use this object to signal when it has read the data
        let hread_event_obj_name = str_to_wstring("ETS2TelemetryReadEvent");

        let hread_event_obj = unsafe {
            OpenEventW(
                0x00100000 | 0x0002,
                BOOL::from(false),
                hread_event_obj_name.as_ptr(),
            )
        };

        // check if operation completed successfully
        windows_get_last_error("OpenEventW - Client Read")?;

        self.hread_event_obj = Some(hread_event_obj);

        // open windows name event object with permission to synchronize and modify,
        // server use this object to signal when it has written data to memory_file
        let hwrite_event_obj_name = str_to_wstring("ETS2TelemetryWriteEvent");

        let hwrite_event_obj = unsafe {
            OpenEventW(
                0x00100000 | 0x0002,
                BOOL::from(false),
                hwrite_event_obj_name.as_ptr(),
            )
        };

        // check if operation completed successfully
        windows_get_last_error("OpenEventW - Server Write")?;

        self.hwrite_event_obj = Some(hwrite_event_obj);

        Ok(())
    }

    pub fn release(&mut self) {
        if self.hmapping_obj.is_some() {
            unsafe { CloseHandle(self.hmapping_obj.take().unwrap()) };
        }

        if self.memory_file_start_address.is_some() {
            unsafe { UnmapViewOfFile(self.memory_file_start_address.take().unwrap()) };
        }

        if self.hmutex_obj.is_some() {
            unsafe { CloseHandle(self.hmutex_obj.take().unwrap()) };
        }

        if self.hread_event_obj.is_some() {
            unsafe { CloseHandle(self.hread_event_obj.take().unwrap()) };
        }

        if self.hwrite_event_obj.is_some() {
            unsafe { CloseHandle(self.hwrite_event_obj.take().unwrap()) };
        }
    }
}

pub struct SharedMemory {
    transmitter: Option<Sender<Box<dyn Input + Send>>>,
    handle: Option<thread::JoinHandle<()>>,
    sentinal: Arc<Mutex<bool>>,
    p_paser: PacketParser,
    selected_game: SelectGame,
}

impl SharedMemory {
    pub fn new(game: SelectGame) -> SharedMemory {
        SharedMemory {
            transmitter: None,
            handle: None,
            sentinal: Arc::new(Mutex::new(false)),
            p_paser: PacketParser::new(game.clone()),
            selected_game: game,
        }
    }
}

impl TelemetryInputMethod for SharedMemory {
    //error not triggered
    fn start(&mut self) -> Result<(), ServiceError> {
        if self.transmitter.is_none() {
            return Err(ServiceError::TransmitterNotSet);
        }

        //copy variable so struct can keep ownership of its members
        let sentinal = Arc::clone(&self.sentinal);
        let p_paser = self.p_paser.clone();
        //let sel_game = self.selected_game.clone();
        let tx = self.transmitter.as_ref().unwrap().clone();

        let handle = thread::spawn(move || {
            let mut ipc = InterProcessCommunication::new();

            match ipc.connect() {
                Err(err) => {
                    println!("failed to init interprocesscommunication {:?}", err);
                }
                Ok(_) => {
                    // create handle array to await for multiple objects
                    let mut wait_handles: [isize; 2] = [0; 2]; //initialize with zeros

                    wait_handles[0] = ipc.hmutex_obj.unwrap().0;
                    wait_handles[1] = ipc.hwrite_event_obj.unwrap().0;

                    // conviences
                    let wait_handles = wait_handles.as_ptr();
                    let base_address = ipc.memory_file_start_address.unwrap();

                    //set loop sentinal value, use to exit infinite loop
                    {
                        *sentinal.lock().unwrap() = true;
                    }

                    loop {
                        //check sentinal condition
                        if !*sentinal.lock().unwrap() {
                            println!("stopping telemetry loop");
                            break;
                        }

                        //blocks until mutex available and server process has signaled read event
                        let dwait_result = unsafe {
                            WaitForMultipleObjects(
                                2,
                                wait_handles,
                                BOOL::from(true),
                                u32::max_value(),
                            )
                        };

                        match dwait_result {
                            //successfull case
                            0x00000000 => {
                                // Reset server process WriteEvent to non-signaled. When execution continues to next iteration the function will block again until
                                // the server process sets the WriteEvent to signaled.
                                let success = unsafe { ResetEvent(ipc.hwrite_event_obj.unwrap()) };
                                if !success.as_bool() {
                                    windows_get_last_error("ResetEvent - write event").unwrap();
                                }

                                // copy packet. plus awareness control loop can stop itself when telemetry broadcaster stops
                                let DataPair(is_alive, packet) = p_paser.data(base_address);

                                match tx.send(packet) {
                                    Err(err) => {
                                        println!("shared memory loop transmit error {}", err);
                                    }
                                    _ => (),
                                }
                                //let (id, type_, time, length) = packet.preview();

                                // println!("is_alive: {} type: {}, id: {}", is_alive, type_, id);

                                // Set client process ReadEvent to signaled. The server process blocks until the client process sets the ReadEvent to signaled before updating
                                // the shared memory with telemetry data
                                let success = unsafe { SetEvent(ipc.hread_event_obj.unwrap()) };
                                if !success.as_bool() {
                                    windows_get_last_error("SetEvent - read event").unwrap();
                                }

                                // Release Mutex so that server process can update shared memory
                                let success = unsafe { ReleaseMutex(ipc.hmutex_obj.unwrap()) };
                                if !success.as_bool() {
                                    windows_get_last_error("ReleaseMutex").unwrap();
                                }

                                // check if loop should exit based on packet paser
                                if !is_alive {
                                    *sentinal.lock().unwrap() = false;
                                }
                            }
                            // all failure cases
                            _ => {
                                println!("failure couldn't aquire all shared memory handles");
                                windows_get_last_error("WaitForMultipleObjects").unwrap();
                            }
                        }
                    }
                }
            }

            //release interprocesscommunicatio handles
            ipc.release();
        });

        self.handle = Some(handle);
        Ok(())
    }
    fn stop(&mut self) {
        *self.sentinal.lock().unwrap() = false;
    }
    fn join(&mut self) {
        // take ownership of handle and join
        self.handle.take().unwrap().join().unwrap();
    }
    fn retrieval_method(&self) -> &str {
        "memory-mapped file"
    }
    fn set_transmitter(&mut self, transmitter: Sender<Box<dyn Input + Send>>) {
        self.transmitter = Some(transmitter);
    }
}
