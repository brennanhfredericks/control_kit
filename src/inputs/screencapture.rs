use crate::{InputProcessMethod, ServiceError};

#[path = "screencapture/desktopduplication.rs"]
mod desktopduplication;
use desktopduplication::DesktopDuplication;

#[path = "screencapture/d3d11device.rs"]
mod d3d11device;
use d3d11device::D3D11Device;

pub struct ScreenCapture {
    screencapture_input: Box<dyn InputProcessMethod + Send>,
}

impl ScreenCapture {
    pub fn via_desktopduplication() -> Result<ScreenCapture, ServiceError> {
        let d_device = match D3D11Device::new() {
            Ok(dev) => dev,
            Err(err) => return Err(ServiceError::WindowsGetLastError(err as i32)),
        };

        let screencapture_input = match DesktopDuplication::new(d_device.get_device()) {
            Ok(dd) => dd,
            Err(err) => return Err(ServiceError::WindowsGetLastError(err as i32)),
        };

        Ok(ScreenCapture {
            screencapture_input: Box::new(screencapture_input),
        })
    }
}
