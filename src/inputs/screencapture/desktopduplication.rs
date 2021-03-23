use crate::telemetry::EventGame;
use crate::{Input, InputProcessMethod, InputType, ServiceError};
mod capture_errors;
use capture_errors::CaptureError;

mod d3d11device;
use d3d11device::{CompatibleCPUTexture2D, D3D11Device};

use winapi::shared::{dxgi1_2, windef, winerror};
use winapi::um::{d3d11, unknwnbase, winuser};
use wio::com::ComPtr;

use std::mem;
use std::ptr;
use std::slice;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

pub struct Pixels {
    pixels: Vec<u8>,
    width: u32,
    height: u32,
}

impl Pixels {
    pub fn new(pixels: Vec<u8>, width: u32, height: u32) -> Pixels {
        Pixels {
            pixels,
            width,
            height,
        }
    }
}

impl Input for Pixels {
    fn input_type(&self) -> InputType {
        InputType::Image
    }
    fn event_type(&self) -> EventGame {
        EventGame::NA
    }

    fn header(&self) -> (u64, u32, u64, u32) {
        (0, 0, 0, 0)
    }
}

pub struct DesktopDuplication {
    //dxgi_device: ComPtr<dxgi1_2::IDXGIDevice2>,
    //dxgi_output: ComPtr<dxgi1_2::IDXGIOutput1>,
    transmitter: Option<Sender<Box<dyn Input + Send>>>,
    handle: Option<thread::JoinHandle<()>>,
    sentinal: Arc<Mutex<bool>>,
}

impl DesktopDuplication {
    pub fn new() -> Result<DesktopDuplication, CaptureError> {
        // get DXGI Device from ID3D11Device

        Ok(DesktopDuplication {
            //dxgi_device,
            //dxgi_output,
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
        let tx = self.transmitter.take().unwrap();

        let handle = thread::spawn(move || {
            // needed to pass pointers between
            // handle error case
            let d3d11device = D3D11Device::new().unwrap();
            let (outdup, dev, devctx) = d3d11device.init_duplication().unwrap();
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
                } else {
                    //release frame before aquiring next
                    println!("desktopduplication loop releaseframe before: ");
                    let success = unsafe { outdup.ReleaseFrame() };
                    println!("desktopduplication loop releaseframe after: {:x}", success);
                    if success != 0x0 {
                        // call will return InvalidCall if frame already release (which is the the case at start)
                        if success != winerror::DXGI_ERROR_INVALID_CALL {
                            // need to be able to restart output duplication api
                            println!("ReleaseFrame Error {:x}", success);
                            break;
                            // return Err(ServiceError::WindowsGetLastError(success));
                        }
                    }
                }

                //println!("desktopduplication loop starting2");

                // aquire new frame
                let success = unsafe {
                    outdup.AcquireNextFrame(1, &mut dxgi_outdupl_frame_info, &mut dxgi_resource)
                };
                println!("desktopduplication loop aquirenextframe:");
                /*
                    println!("desktopduplication loop starting3");
                    if success != 0x0 {
                        // need to be able to restart output duplication api
                        println!("AquireFrame Error {:x}", success);
                        return Err(ServiceError::WindowsGetLastError(success));
                    }

                    if dxgi_outdupl_frame_info.AccumulatedFrames < 1 {
                        //no frame available wait before retrying
                        println!("No accumalated frames");
                        thread::sleep(Duration::from_millis(2));

                        continue;
                    }

                    let dxgi_resource = unsafe { ComPtr::from_raw(dxgi_resource) };

                    let gpu_texture: ComPtr<d3d11::ID3D11Texture2D> = match dxgi_resource.cast() {
                        Ok(texture) => texture,
                        Err(err) => {
                            println!("ID3D11Texture2D Error {:x}", err);
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
                        devicecontext.CopyResource(
                            gpu_texture.as_raw() as *mut d3d11::ID3D11Resource,
                            cpu_texture.as_raw() as *mut d3d11::ID3D11Resource,
                        )
                    }

                    let success = unsafe {
                        devicecontext.Map(
                            cpu_texture.as_raw() as *mut d3d11::ID3D11Resource,
                            subresource,
                            d3d11::D3D11_MAP_READ,
                            0,
                            &mut mapped_resource,
                        )
                    };

                    if success != 0x0 {
                        println!("Map Error {:x}", success);
                        return Err(ServiceError::WindowsGetLastError(success));
                    }

                    let byte_size = |x| x * mem::size_of::<u8>() / mem::size_of::<u8>();

                    let stride = mapped_resource.RowPitch as usize / mem::size_of::<u8>();
                    let byte_stride = byte_size(stride);

                    let buf = unsafe {
                        slice::from_raw_parts(
                            mapped_resource.pData as *const u8,
                            byte_stride * height as usize,
                        )
                    };

                    let pixels = Pixels::new(buf.to_vec(), width, height);

                    match tx.send(Box::new(pixels)) {
                        Err(err) => {
                            println!("desktopduplication loop transmit error {}", err);
                        }
                        _ => (),
                    }

                    last_frame = Instant::now();

                    unsafe {
                        devicecontext.Unmap(
                            cpu_texture.as_raw() as *mut d3d11::ID3D11Resource,
                            subresource,
                        )
                    };

                    println!("got new frame");
                }

                // if let Err(err) = res {
                //     println!("error in desktop application loop {:?}", err)
                // }

                let success = unsafe { output_duplication.Release() };

                if success != 0x0 {
                    println!("OuputDuplication Release Error {:x}", success);
                    return Err(ServiceError::WindowsGetLastError(success as i32));
                }

                Ok(())
                */
            }
        });

        self.handle = Some(handle);

        println!("handle desktopduplication");
        Ok(())
    }
    fn stop(&mut self) {
        //stop loop
        println!("setting conditional to false");
        *self.sentinal.lock().unwrap() = false;
        //println!("set conditional to false");
    }
    fn join(&mut self) {
        // take ownership of handle and
        println!("joining desktopduplication");
        let res = self.handle.take().unwrap().join();

        // let res_outer = match res.unwrap() {
        //     Ok(inner) => inner,
        //     Err(e) => {
        //         println!("{:?}", e);
        //     }
        //};
    }
    fn method(&self) -> &str {
        "DesktopDuplicationAPI"
    }
    fn set_transmitter(&mut self, transmitter: Sender<Box<dyn Input + Send>>) {
        self.transmitter = Some(transmitter);
    }
}
