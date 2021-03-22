use control::screencapture::ScreenCapture;
use control::synchronization::Synchronization;
use control::telemetry::{SelectGame, Telemetry};
use control::{ServiceType, Services};
use std::io::{BufRead, BufReader, Error, ErrorKind};
use std::process::{Command, Stdio};
use std::rc::Rc;
use std::sync::mpsc::channel;
use std::thread;
use std::time::{Duration, Instant};

type CResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> CResult<()> {
    println!("main run");

    let (out_transmitter, out_receiver) = channel();

    //game selection
    //let sel_game = SelectGame::ETS2;
    // setup telemetry

    //let mut sync = Synchronization::new();

    //get transmitter for synchronization services
    //let tx_telemetry = sync.get_input_transmitter();
    //let tx_screencapture = sync.get_input_transmitter();

    //set transmitter for syncing broadcast service
    //sync.set_output_transmitter(out_transmitter);

    //create telemetry serivce
    //let mut ets2_telemetry = Telemetry::via_shared_memory(sel_game);
    //ets2_telemetry.set_transmitter(tx_telemetry);

    //crate screencapture service
    let mut dd_screencapture = ScreenCapture::via_desktopduplication().unwrap();
    dd_screencapture.set_transmitter(out_transmitter);

    let mut cap_sess = Services::new();

    // start sync services
    // cap_sess
    //     .add_service(ServiceType::SynchronizeInputs, Box::new(sync))
    //     .unwrap();

    cap_sess
        .add_service(ServiceType::ScreenCaptureInput, Box::new(dd_screencapture))
        .unwrap();

    // start telemetry emulation thread
    // let mut emulation_thread = Command::new(".\\tests.\\TelemetryEmulation.exe")
    //     .stdout(Stdio::null()) //ignore stream, otherwise windows blocks
    //     .spawn()
    //     .unwrap();

    //wait for telemetry emulation to setup and complete
    //thread::sleep(Duration::from_secs(1));

    //start telemetry services
    // cap_sess
    //     .add_service(ServiceType::TelemetryInput, Box::new(ets2_telemetry))
    //     .unwrap();

    // wait till telemetry is done
    //cap_sess.block_until_telemetry_finished().unwrap();

    thread::sleep(Duration::from_secs(2));
    println!("done sleep for t");

    // stop all running service
    cap_sess.stop_all_services().unwrap();

    //join emulation thread
    //emulation_thread.wait().unwrap();

    // for i in out_receiver.iter() {
    //     assert!(i[0].header().0 + 1 == i.last().unwrap().header().0);
    //     // println!("f - id: {}", i[0].header().0);
    //     // println!("l - id: {}", i.last().unwrap().header().0);
    // }

    Ok(())
}
