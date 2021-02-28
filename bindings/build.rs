fn main() {
    windows::build!(
        windows::win32::system_services::{
            MapViewOfFile, SetEvent, UnmapViewOfFile,ResetEvent,ReleaseMutex,
            WaitForMultipleObjects,OpenFileMappingW,OpenMutexW,OpenEventW,
        },
        windows::win32::windows_programming::CloseHandle,
        windows::win32::debug::GetLastError,

    );
}
