//use std::rc::Rc;
use std::ptr;
use winapi::um::{d3d11, d3dcommon};
use wio::com::ComPtr;

use crate::windows_get_last_error;
//create d3d11 device that will be used to process captured images.
#[path = "capture_errors.rs"]
mod capture_errors;
use capture_errors::CaptureError;
pub struct D3D11Device {
    device: ComPtr<d3d11::ID3D11Device>,
    devicecontext: ComPtr<d3d11::ID3D11DeviceContext>,
}

impl D3D11Device {
    pub fn new() -> Result<D3D11Device, CaptureError> {
        let (mut device, mut devicecontext) = (ptr::null_mut(), ptr::null_mut());

        let success = unsafe {
            d3d11::D3D11CreateDevice(
                ptr::null_mut(), // A pointer to the adapter to be use when creating a device, pass NULL to use the default adapter
                d3dcommon::D3D_DRIVER_TYPE_HARDWARE,
                ptr::null_mut(), // A handle to a DLL that implements a software rasterizaer. NA using hardware
                d3d11::D3D11_CREATE_DEVICE_DEBUG,
                ptr::null_mut(), // Use default array of feature levels
                0,
                d3d11::D3D11_SDK_VERSION,
                &mut device, // wants pointer to pointer
                ptr::null_mut(),
                &mut devicecontext,
            )
        };

        if success != 0x0 {
            //add log unable to d3d11 create device
            return Err(CaptureError::from_win_error(success));
        }

        let device: ComPtr<d3d11::ID3D11Device> = unsafe { ComPtr::from_raw(device) };
        let devicecontext: ComPtr<d3d11::ID3D11DeviceContext> =
            unsafe { ComPtr::from_raw(devicecontext) };

        Ok(D3D11Device {
            device,
            devicecontext,
        })
    }

    pub fn get_device(&self) -> &ComPtr<d3d11::ID3D11Device> {
        &self.device
    }

    pub fn get_device_context(&self) -> &ComPtr<d3d11::ID3D11DeviceContext> {
        &self.devicecontext
    }
}

struct CompatibleCPUTexture2D;

impl CompatibleCPUTexture2D {
    fn create(device: &ComPtr<d3d11::ID3D11Device>, src: &ComPtr<d3d11::ID3D11Texture2D>) {}
}
