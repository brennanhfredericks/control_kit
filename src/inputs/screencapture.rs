use crate::{Input, InputProcessMethod, Process, ServiceError};
use std::sync::mpsc::Sender;

//use dxgcap::DXGIManager;

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

        //let d = *d_device.get_device();
        let screencapture_input =
            match DesktopDuplication::new(d_device.get_device(), d_device.get_device_context()) {
                Ok(dd) => dd,
                Err(err) => return Err(ServiceError::WindowsGetLastError(err as i32)),
            };

        Ok(ScreenCapture {
            screencapture_input: Box::new(screencapture_input),
        })
    }

    pub fn get_method(&self) -> &str {
        self.screencapture_input.method()
    }

    pub fn set_transmitter(&mut self, transmitter: Sender<Box<dyn Input + Send>>) {
        self.screencapture_input.set_transmitter(transmitter);
    }
}

impl Process for ScreenCapture {
    fn start(&mut self) -> Result<(), ServiceError> {
        self.screencapture_input.start()?;
        Ok(())
    }

    fn stop(&mut self) -> Result<(), ServiceError> {
        self.screencapture_input.stop();

        Ok(())
    }

    fn join(&mut self) {
        self.screencapture_input.join();
    }
}
