use bindings::{
    windows::win32::system_services::{
        MapViewOfFile, OpenEventW, OpenFileMappingW, OpenMutexW, ReleaseMutex, ResetEvent,
        SetEvent, UnmapViewOfFile, WaitForMultipleObjects, HANDLE,
    },
    windows::win32::windows_programming::CloseHandle,
    windows::BOOL,
};

use crate::{str_to_wstring, windows_get_last_error, Input, InputType, Process, ServiceError};
use std::ffi::c_void;
use std::mem;
use std::result::Result;
use std::slice;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::telemetry::TelemetryInputMethod;
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
}

impl SharedMemory {
    pub fn new() -> SharedMemory {
        SharedMemory {
            transmitter: None,
            handle: None,
            sentinal: Arc::new(Mutex::new(false)),
        }
    }
}

impl TelemetryInputMethod for SharedMemory {
    fn start(&mut self) -> Result<(), ServiceError> {
        Ok(())
    }
    fn stop(&mut self) -> Result<(), ServiceError> {
        Ok(())
    }
    fn join(&self) {}
    fn retrieval_method(&self) -> &str {
        "memory-mapped file"
    }
    fn set_transmitter(&mut self, transmitter: Sender<Box<dyn Input + Send>>) {
        self.transmitter = Some(transmitter);
    }
}
