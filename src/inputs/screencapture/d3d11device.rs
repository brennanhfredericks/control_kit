//use std::rc::Rc;
use winpai::um::{d3d11, d3dcommon};
use wio::com;

//create d3d11 device that will be used to process captured images.

pub struct D3D11Device {
    device: com::ComPtr<d3d11::ID3D11Device>,
    devicecontext: com::ComPtr<d3d11::ID3D11DeviceContext>,
}
