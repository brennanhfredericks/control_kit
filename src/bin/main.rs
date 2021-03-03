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
    let sel_game = SelectGame::ets2;
    // setup telemetry

    let sync = Synchronization::new();

    let tx = sync.get_transmitter();

    let mut ets2_telemetry = Telemetry::via_shared_memory(sel_game);

    ets2_telemetry.set_transmitter(tx);

    let mut cap_sess = Services::new();

    // start sync services
    // cap_sess
    //     .add_service(ServiceType::synchronize_inputs, Box::new(sync))
    //     .unwrap();

    cap_sess
        .add_service(ServiceType::telemetry_input, Box::new(ets2_telemetry))
        .unwrap();

    Ok(())
}
