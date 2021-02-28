use bindings::{
    windows::win32::debug::GetLastError,
    windows::win32::system_services::{
        MapViewOfFile, OpenEventW, OpenFileMappingW, OpenMutexW, ReleaseMutex, ResetEvent,
        SetEvent, UnmapViewOfFile, WaitForMultipleObjects, HANDLE,
    },
    windows::win32::windows_programming::CloseHandle,
    windows::BOOL,
};

use std::ffi::c_void;
use std::mem;
use std::result::Result;
use std::slice;
use std::sync::{Arc, Mutex};
//use std::thread;
use crate::{Process, ServiceError};
//use to setup windows inter process communication and sychronization objects
struct InterProcessCommunication {
    hmapping_obj: None,
    memory_file_start_address: Option<*mut c_void>,
    hmutex_obj: None,
    hread_event_obj: None,
    hwrite_event_obj: None,
}

impl InterProcessCommunication {
    pub fn new() {}

    pub fn connect() {}

    pub fn release() {}
}

pub struct SharedMemory {}

impl SharedMemory {}

impl Process for SharedMemory {
    fn start(&mut self) -> Result<(), ServiceError> {}
    fn stop(&mut self) -> Result<(), ServiceError> {}
    fn join(mut self) {}
}
