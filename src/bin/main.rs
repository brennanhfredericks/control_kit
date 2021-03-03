use control::synchronization::Synchronization;
use control::telemetry::{SelectGame, Telemetry};
use control::{ServiceType, Services};
use std::io::{BufRead, BufReader, Error, ErrorKind};
use std::process::{Command, Stdio};
use std::rc::Rc;
use std::thread;
use std::time::{Duration, Instant};

type CResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> CResult<()> {
    println!("main run");

    // test only service start stop and telementry

    //game selection
    let sel_game = SelectGame::ETS2;
    // setup telemetry

    let sync = Synchronization::new();

    let tx = sync.get_transmitter();

    let mut ets2_telemetry = Telemetry::via_shared_memory(sel_game);

    ets2_telemetry.set_transmitter(tx);

    let mut cap_sess = Services::new();

    // start sync services
    cap_sess
        .add_service(ServiceType::SynchronizeInputs, Box::new(sync))
        .unwrap();

    // start telemetry emulation thread
    let mut emulation_thread = Command::new(".\\tests.\\TelemetryEmulation.exe")
        .spawn()
        .unwrap();

    //wait for telemetry emulation to setup and complete
    thread::sleep(Duration::from_secs(1));

    //start telemetry services
    cap_sess
        .add_service(ServiceType::TelemetryInput, Box::new(ets2_telemetry))
        .unwrap();

    // wait till telemetry is done
    cap_sess.block_until_telemetry_finished().unwrap();

    // stop all running service
    cap_sess.stop_all_services().unwrap();

    //join emulation thread
    emulation_thread.wait().unwrap();

    Ok(())
}
