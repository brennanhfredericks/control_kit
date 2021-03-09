use winapi::shared::winerror;
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CaptureError {
    ZeroAccumalatedFrames, // no new frames available
    Cast,                  // casting error
    AccessLost,            // winerror::DXGI_ERROR_ACCESS_LOST,
    DeviceRemoved,         // winerror::DXGI_ERROR_DEVICE_REMOVED,
    ExclusiveOwnership,    // winerror::DXGI_ERROR_GRAPHICS_VIDPN_SOURCE_IN_USE,
    InvalidCall,           // winerror::DXGI_ERROR_INVALID_CALL,
    NotFound,              // winerror::DXGI_ERROR_NOT_FOUND,
    WaitTimeout,           // winerror::DXGI_ERROR_WAIT_TIMEOUT,
    WasStillDrawing,       // winerror::DXGI_ERROR_WAS_STILL_DRAWING,
    CompatibleTexture2D,   // could not create compatible CPU texture2d to access data,
    DxgiError,             // fall throught to cover other errors
    JpegEncoder,           // could not encode raw bytes as jpeg bytes buffer
    JpegBuffer,            // could not get jpeg buffer (Vec<u8>)
    NoDebugLayer,
    InvalidParameter,
}

impl CaptureError {
    /// maps DXGI_ERROR to CaptureError
    pub fn from_win_error(hr: i32) -> CaptureError {
        match hr {
            winerror::DXGI_ERROR_ACCESS_LOST => CaptureError::AccessLost,
            winerror::DXGI_ERROR_DEVICE_REMOVED => CaptureError::DeviceRemoved,
            winerror::DXGI_ERROR_GRAPHICS_VIDPN_SOURCE_IN_USE => CaptureError::ExclusiveOwnership,
            winerror::DXGI_ERROR_INVALID_CALL => CaptureError::InvalidCall,
            winerror::DXGI_ERROR_NOT_FOUND => CaptureError::NotFound,
            winerror::DXGI_ERROR_WAIT_TIMEOUT => CaptureError::WaitTimeout,
            winerror::DXGI_ERROR_WAS_STILL_DRAWING => CaptureError::WasStillDrawing,
            winerror::E_FAIL => CaptureError::NoDebugLayer,
            winerror::E_INVALIDARG => CaptureError::InvalidParameter,
            _ => CaptureError::DxgiError,
        }
    }
}
