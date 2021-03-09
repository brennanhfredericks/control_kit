//use std::rc::Rc;
use std::ptr;
use winapi::um::{d3d11, d3dcommon};
use wio::com;

use crate::windows_get_last_error;
//create d3d11 device that will be used to process captured images.

pub struct D3D11Device {
    device: com::ComPtr<d3d11::ID3D11Device>,
    devicecontext: com::ComPtr<d3d11::ID3D11DeviceContext>,
}

impl D3D11Device {
    pub fn new() -> D3D11Device {
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

        if success < 0 {
            windows_get_last_error("D311CreateDevice").unwrap();
        }

        let device: com::ComPtr<d3d11::ID3D11Device> = unsafe { com::ComPtr::from_raw(device) };
        let devicecontext: com::ComPtr<d3d11::ID3D11DeviceContext> =
            unsafe { com::ComPtr::from_raw(devicecontext) };

        D3D11Device {
            device,
            devicecontext,
        }
    }

    pub fn get_device(&self) -> &com::ComPtr<d3d11::ID3D11Device> {
        &self.device
    }

    pub fn get_device_context(&self) -> &com::ComPtr<d3d11::ID3D11DeviceContext> {
        &self.devicecontext
    }
}
