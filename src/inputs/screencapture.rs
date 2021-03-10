use crate::{InputProcessMethod, ServiceError};

#[path = "screencapture/desktopduplication.rs"]
mod desktopduplication;
use desktopduplication::DesktopDuplication;

pub struct ScreenCapture {
    screencapture_input: Box<dyn InputProcessMethod + Send>,
}

impl ScreenCapture {
    pub fn via_desktopduplication() -> () {
        // ScreenCapture {
        //     screencapture_input: Box::new(DesktopDuplication::new()),
        // }
    }
}
