//use std::rc::Rc;
use std::sync::Arc;
use std::{mem, ptr};
use winapi::shared::dxgiformat;
use winapi::um::{d3d11, d3dcommon};
use wio::com::ComPtr;

//create d3d11 device that will be used to process captured images.
#[path = "capture_errors.rs"]
mod capture_errors;
use capture_errors::CaptureError;
pub struct D3D11Device {
    device: Arc<ComPtr<d3d11::ID3D11Device>>,
    devicecontext: Arc<ComPtr<d3d11::ID3D11DeviceContext>>,
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
            device: Arc::new(device),
            devicecontext: Arc::new(devicecontext),
        })
    }

    pub fn get_device(&self) -> Arc<ComPtr<d3d11::ID3D11Device>> {
        Arc::clone(&self.device)
    }

    pub fn get_device_context(&self) -> Arc<ComPtr<d3d11::ID3D11DeviceContext>> {
        Arc::clone(&self.devicecontext)
    }
}

pub struct CompatibleCPUTexture2D;

impl CompatibleCPUTexture2D {
    pub fn create(
        device: &ComPtr<d3d11::ID3D11Device>,
        src: &ComPtr<d3d11::ID3D11Texture2D>,
    ) -> Result<(ComPtr<d3d11::ID3D11Texture2D>, u32, u32), CaptureError> {
        let mut cputex: *mut d3d11::ID3D11Texture2D = ptr::null_mut();
        let mut src_desc: d3d11::D3D11_TEXTURE2D_DESC = unsafe { mem::zeroed() };

        //get description of source texture
        unsafe {
            src.GetDesc(&mut src_desc);
        }

        let mut dest_desc: d3d11::D3D11_TEXTURE2D_DESC = unsafe { mem::zeroed() };

        // setup destination texture
        dest_desc.Width = src_desc.Width;
        dest_desc.Height = src_desc.Height;
        dest_desc.MipLevels = src_desc.MipLevels;
        dest_desc.SampleDesc = src_desc.SampleDesc;
        dest_desc.ArraySize = src_desc.ArraySize;
        dest_desc.Format = dxgiformat::DXGI_FORMAT_B8G8R8A8_UNORM;
        dest_desc.CPUAccessFlags = d3d11::D3D11_CPU_ACCESS_READ;
        dest_desc.Usage = d3d11::D3D11_USAGE_STAGING;

        // create cpu texture
        let success = unsafe { device.CreateTexture2D(&dest_desc, ptr::null_mut(), &mut cputex) };

        if success != 0x0 {
            //add log unable to d3d11 create device
            return Err(CaptureError::from_win_error(success));
        }

        let cputex = unsafe { ComPtr::from_raw(cputex) };
        Ok((cputex, dest_desc.Width.clone(), dest_desc.Height.clone()))
    }
}
