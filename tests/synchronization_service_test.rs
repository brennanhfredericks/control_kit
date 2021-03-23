use control::screencapture::ScreenCapture;
use control::synchronization::Synchronization;
use control::telemetry::{EventGame, SelectGame, Telemetry};
use control::{ServiceType, Services};
use std::sync::mpsc::channel;
use std::thread;
use std::time::{Duration, Instant};

mod emulation_start;

use emulation_start::ETS2Emulation;
use std::io::BufRead;

#[test]
fn synchronization_service_groupify_first_last_type_check() {
    let (out_transmitter, out_receiver) = channel();

    //game selection
    let sel_game = SelectGame::ETS2;
    // setup telemetry

    let mut sync = Synchronization::new();

    let tx_telemetry = sync.get_input_transmitter();
    let tx_screencapture = sync.get_input_transmitter();
    sync.set_output_transmitter(out_transmitter);

    let mut ets2_telemetry = Telemetry::via_shared_memory(sel_game);
    let mut dd_screencapture = match ScreenCapture::via_desktopduplication() {
        Ok(sc) => sc,
        Err(err) => {
            println!("screen capture: {:?}", err);
            panic!("screen capture did not start");
        }
    };

    ets2_telemetry.set_transmitter(tx_telemetry);
    dd_screencapture.set_transmitter(tx_screencapture);

    let mut cap_sess = Services::new();

    // start sync services
    cap_sess
        .add_service(ServiceType::SynchronizeInputs, Box::new(sync))
        .unwrap();

    // start telemetry emulation thread
    let mut emuprocess = ETS2Emulation::start();

    //wait for telemetry emulation to setup and complete
    thread::sleep(Duration::from_secs(1));

    //start telemetry services
    cap_sess
        .add_service(ServiceType::TelemetryInput, Box::new(ets2_telemetry))
        .unwrap();

    // start screen capture services
    cap_sess
        .add_service(ServiceType::ScreenCaptureInput, Box::new(dd_screencapture))
        .unwrap();

    // wait till telemetry is done
    cap_sess.block_until_telemetry_finished().unwrap();

    // stop all running service
    cap_sess.stop_all_services().unwrap();

    if let Err(e) = emuprocess.kill() {
        println!("error killing child process: {:?}", e);
    }
    for i in out_receiver.iter() {
        println!("size: {}", i.len());
        assert!(i.first().unwrap().event_type() == EventGame::FrameStartEvent);
        assert!(i.last().unwrap().event_type() == EventGame::FrameEndEvent);
    }
}
