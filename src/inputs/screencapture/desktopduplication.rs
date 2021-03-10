mod d3d11device;

use d3d11device::D3D11Device;

use crate::InputProcessMethod;

mod capture_errors;
use capture_errors::CaptureError;

use winapi::shared::dxgi1_2;
use winapi::um::d3d11;
use wio::com::ComPtr;

use std::ptr;

pub struct DesktopDuplication {
    dxgi_device: ComPtr<dxgi1_2::IDXGIDevice2>,
    dxgi_output: ComPtr<dxgi1_2::IDXGIOutput1>,
    //devicecontext: &ComPtr<d3d11::ID3D11DeviceContext>,
}

impl DesktopDuplication {
    pub fn new(
        device: &ComPtr<d3d11::ID3D11Device>,
        devicecontext: &ComPtr<d3d11::ID3D11DeviceContext>,
    ) -> Result<DesktopDuplication, CaptureError> {
        // get DXGI Device from ID3D11Device
        let dxgi_device: ComPtr<dxgi1_2::IDXGIDevice2> = match device.cast() {
            Ok(dev) => dev,
            Err(err) => {
                // add logging
                return Err(CaptureError::from_win_error(err));
            }
        };

        //get DXGI Adapter from  DXGI Device, use to retrieve all outputs
        let mut dxgi_adapter = ptr::null_mut();

        let success = unsafe { dxgi_device.GetAdapter(&mut dxgi_adapter) };

        //check if operation complete succefully
        if success != 0x0 {
            // add logging
            return Err(CaptureError::from_win_error(success));
        }

        // create ComPtr from raw pointer
        let dxgi_adapter = unsafe { ComPtr::from_raw(dxgi_adapter) };

        // use to primary monitor. multiple monitor require vector
        let mut dxgi_output = ptr::null_mut();

        //use DXGI Adapter to retrieve primary monitor (is at index zero)
        let success = unsafe { dxgi_adapter.EnumOutputs(0, &mut dxgi_output) };

        if success != 0x0 {
            //add logging
            return Err(CaptureError::from_win_error(success));
        }

        let dxgi_output = unsafe { ComPtr::from_raw(dxgi_output) };

        // cast DXGI Output to  DXGI Output1 to access duplication functionality
        let dxgi_output: ComPtr<dxgi1_2::IDXGIOutput1> = match dxgi_output.cast() {
            Ok(out) => out,
            Err(err) => {
                //add logging
                return Err(CaptureError::from_win_error(err));
            }
        };

        Ok(DesktopDuplication {
            dxgi_device,
            dxgi_output,
        })
    }
}
