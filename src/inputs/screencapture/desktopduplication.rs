use crate::{Input, InputProcessMethod, ServiceError};

mod capture_errors;
use capture_errors::CaptureError;

use winapi::shared::dxgi1_2;
use winapi::um::{d3d11, unknwnbase};
use wio::com::ComPtr;

use std::ptr;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread;

pub struct DesktopDuplication {
    dxgi_device: ComPtr<dxgi1_2::IDXGIDevice2>,
    dxgi_output: ComPtr<dxgi1_2::IDXGIOutput1>,
    dxgi_output_duplication: Option<ComPtr<dxgi1_2::IDXGIOutputDuplication>>,
    devicecontext: Option<ComPtr<d3d11::ID3D11DeviceContext>>, //needed to copy data between textures
    transmitter: Option<Sender<Box<dyn Input + Send>>>,
    handle: Option<thread::JoinHandle<Result<(), ServiceError>>>,
    sentinal: Arc<Mutex<bool>>,
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

        let mut dxgi_out_dup = ptr::null_mut();

        let success = unsafe {
            dxgi_output.DuplicateOutput(
                dxgi_device.as_raw() as *mut unknwnbase::IUnknown,
                &mut dxgi_out_dup,
            )
        };

        if success != 0x0 {
            //add error log
            return Err(CaptureError::from_win_error(success));
        }

        let dxgi_output_duplication = unsafe { ComPtr::from_raw(dxgi_out_dup) };

        Ok(DesktopDuplication {
            dxgi_device,
            dxgi_output,
            dxgi_output_duplication: Some(dxgi_output_duplication),
            devicecontext: Some(devicecontext.clone()),
            transmitter: None,
            handle: None,
            sentinal: Arc::new(Mutex::new(false)),
        })
    }
}

unsafe impl std::marker::Send for DesktopDuplication {} // Send trait implemented manual have to test

impl InputProcessMethod for DesktopDuplication {
    fn start(&mut self) -> Result<(), ServiceError> {
        if self.transmitter.is_none() {
            return Err(ServiceError::TransmitterNotSet);
        }

        // clone variable
        let sentinal = Arc::clone(&self.sentinal);

        // take value
        let device_context = self.devicecontext.take().unwrap().as_raw() as usize;
        let tx = self.transmitter.take().unwrap();
        let output_duplication = self.dxgi_output_duplication.take().unwrap().as_raw() as usize;
        let handle = thread::spawn(move || {
            // needed to pass pointers between threads
            let output_duplication: ComPtr<dxgi1_2::IDXGIOutputDuplication> =
                unsafe { ComPtr::from_raw(output_duplication as *mut _) };

            let device_context: ComPtr<d3d11::ID3D11DeviceContext> =
                unsafe { ComPtr::from_raw(device_context as *mut _) };
            {
                *sentinal.lock().unwrap() = true;
            }

            loop {
                //check sentinal condition
                if !*sentinal.lock().unwrap() {
                    println!("stopping desktopduplication loop");
                    break;
                }
            }

            let success = unsafe { output_duplication.Release() };

            if success != 0x0 {
                return Err(ServiceError::WindowsGetLastError(success as i32));
            }

            Ok(())
        });
        self.handle = Some(handle);
        Ok(())
    }
    fn stop(&mut self) {}
    fn join(&mut self) {}
    fn method(&self) -> &str {
        "DesktopDuplicationAPI"
    }
    fn set_transmitter(&mut self, transmitter: Sender<Box<dyn Input + Send>>) {}
}
