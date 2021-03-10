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
            Err(_) => return Err(ServiceError::FailedToInitialize),
        };

        let screencapture_input =
            match DesktopDuplication::new(d_device.get_device(), d_device.get_device_context()) {
                Ok(dd) => dd,
                Err(_) => return Err(ServiceError::FailedToInitialize),
            };

        Ok(ScreenCapture {
            screencapture_input: Box::new(screencapture_input),
        })
    }
}
