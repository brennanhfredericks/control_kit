use crate::{Input, InputProcessMethod, ServiceError};

mod capture_errors;
use capture_errors::CaptureError;

mod d3d11device;
use d3d11device::CompatibleCPUTexture2D;

use winapi::shared::{dxgi1_2, winerror};
use winapi::um::{d3d11, unknwnbase};
use wio::com::ComPtr;

use std::mem;
use std::ptr;
use std::slice;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

pub struct DesktopDuplication {
    //dxgi_device: ComPtr<dxgi1_2::IDXGIDevice2>,
    //dxgi_output: ComPtr<dxgi1_2::IDXGIOutput1>,
    dxgi_output_duplication: Option<ComPtr<dxgi1_2::IDXGIOutputDuplication>>,
    device: Option<ComPtr<d3d11::ID3D11Device>>, //needed to copy data between textures
    transmitter: Option<Sender<Box<dyn Input + Send>>>,
    handle: Option<thread::JoinHandle<Result<(), ServiceError>>>,
    sentinal: Arc<Mutex<bool>>,
}

impl DesktopDuplication {
    pub fn new(device: &ComPtr<d3d11::ID3D11Device>) -> Result<DesktopDuplication, CaptureError> {
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
            //dxgi_device,
            //dxgi_output,
            dxgi_output_duplication: Some(dxgi_output_duplication),
            device: Some(device.clone()),
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

        //need to add capture interval,

        // clone variable
        let sentinal = Arc::clone(&self.sentinal);

        // take value
        let device = self.device.take().unwrap().as_raw() as usize;
        let tx = self.transmitter.take().unwrap();
        let output_duplication = self.dxgi_output_duplication.take().unwrap().as_raw() as usize;

        let handle = thread::spawn(move || {
            // needed to pass pointers between threads
            let mut output_duplication: ComPtr<dxgi1_2::IDXGIOutputDuplication> =
                unsafe { ComPtr::from_raw(output_duplication as *mut _) };

            let device: ComPtr<d3d11::ID3D11Device> = unsafe { ComPtr::from_raw(device as *mut _) };

            let mut device_context = ptr::null_mut();

            unsafe { device.GetImmediateContext(&mut device_context) };

            let device_context = unsafe { ComPtr::from_raw(device_context) };

            {
                *sentinal.lock().unwrap() = true;
            }

            let mut compatible_texture: Option<(ComPtr<d3d11::ID3D11Texture2D>, u32, u32)> = None;
            let mut dxgi_outdupl_frame_info: dxgi1_2::DXGI_OUTDUPL_FRAME_INFO =
                unsafe { mem::zeroed() };
            let mut mapped_resource: d3d11::D3D11_MAPPED_SUBRESOURCE = unsafe { mem::zeroed() };
            let subresource = d3d11::D3D11CalcSubresource(0, 0, 0);

            let mut dxgi_resource = ptr::null_mut();

            let mut last_frame = Instant::now();
            let mut first_iter = true;

            loop {
                //need to be able to recreate output duplication if failed, investigate how

                let timestamp = Instant::now();
                //check sentinal condition
                if !*sentinal.lock().unwrap() {
                    println!("stopping desktopduplication loop");
                    break;
                }

                // check if interval has passed and not first iteration
                if timestamp.duration_since(last_frame).as_millis() < 15 && !first_iter {
                    continue;
                }

                // use to capture at time zero or close to it
                if first_iter {
                    first_iter = false;
                }

                //release frame before aquiring next frame
                let success = unsafe { output_duplication.ReleaseFrame() };

                if success != 0x0 {
                    // call will return InvalidCall if frame already release (which is the the case at start)
                    if success != winerror::DXGI_ERROR_INVALID_CALL {
                        // need to be able to restart output duplication api
                        return Err(ServiceError::WindowsGetLastError(success));
                    }
                }

                // aquire new frame
                let success = unsafe {
                    output_duplication.AcquireNextFrame(
                        1,
                        &mut dxgi_outdupl_frame_info,
                        &mut dxgi_resource,
                    )
                };

                if success != 0x0 {
                    // need to be able to restart output duplication api
                    return Err(ServiceError::WindowsGetLastError(success));
                }

                if dxgi_outdupl_frame_info.AccumulatedFrames < 1 {
                    //no frame available wait before retrying
                    thread::sleep(Duration::from_millis(2));
                    continue;
                }

                let dxgi_resource = unsafe { ComPtr::from_raw(dxgi_resource) };

                let gpu_texture: ComPtr<d3d11::ID3D11Texture2D> = match dxgi_resource.cast() {
                    Ok(texture) => texture,
                    Err(err) => {
                        return Err(ServiceError::WindowsGetLastError(err));
                    }
                };

                // get cpu compatible texture
                if compatible_texture.is_none() {
                    compatible_texture = match CompatibleCPUTexture2D::create(&device, &gpu_texture)
                    {
                        Ok(par) => Some(par),
                        Err(f) => {
                            return Err(ServiceError::WindowsGetLastError(f as i32));
                        }
                    };
                }

                let (cpu_texture, width, height) = compatible_texture.as_ref().unwrap().clone();

                // copy texture from GPU texture to CPU texture
                unsafe {
                    device_context.CopyResource(
                        gpu_texture.as_raw() as *mut d3d11::ID3D11Resource,
                        cpu_texture.as_raw() as *mut d3d11::ID3D11Resource,
                    )
                }

                let success = unsafe {
                    device_context.Map(
                        cpu_texture.as_raw() as *mut d3d11::ID3D11Resource,
                        subresource,
                        d3d11::D3D11_MAP_READ,
                        0,
                        &mut mapped_resource,
                    )
                };

                if success != 0x0 {
                    return Err(ServiceError::WindowsGetLastError(success));
                }

                let byte_size = |x| x * mem::size_of::<u8>() / mem::size_of::<u8>();

                let stride = mapped_resource.RowPitch as usize / mem::size_of::<u8>();
                let byte_stride = byte_size(stride);

                let pixel_buf = unsafe {
                    slice::from_raw_parts(
                        mapped_resource.pData as *const u8,
                        byte_stride * height as usize,
                    )
                };

                let pixel_buf = pixel_buf.to_vec();

                last_frame = Instant::now();

                unsafe {
                    device_context.Unmap(
                        cpu_texture.as_raw() as *mut d3d11::ID3D11Resource,
                        subresource,
                    )
                };
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
