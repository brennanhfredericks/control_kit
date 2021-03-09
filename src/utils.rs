use crate::ServiceError;
use bindings::windows::win32::debug::GetLastError;

pub fn str_to_wstring(name: &str) -> Vec<u16> {
    let mut wstring: Vec<u16> = String::from(name).encode_utf16().collect();
    wstring.push(0);
    wstring
}

pub fn windows_get_last_error(debug: &str) -> Result<(), ServiceError> {
    let error: i32 = unsafe { GetLastError() as i32 };

    if error != 0x0 {
        // error is the original windows api error, use log and debug str
        println!("Error calling windows api: {} : {}", debug, error);
        return Err(ServiceError::WindowsGetLastError(error));
    }

    Ok(())
}
