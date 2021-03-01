use control::telemetry::{SelectGame, Telemetry};
use control::{ServiceType, Services};
use std::thread;
use std::time::Duration;

fn main() {
    println!("main test");

    // test only service start stop and telementry

    //game selection
    let sel_game = SelectGame::ets2;
    // setup telemetry
    let ets2_telemetry = Telemetry::via_shared_memory(sel_game);

    let mut cap_sess = Services::new();

    // start test exe

    let res = cap_sess
        .add_service(ServiceType::telemetry_input, Box::new(ets2_telemetry))
        .unwrap();
    thread::sleep(Duration::from_secs(1));
    cap_sess.stop_all_services().unwrap();
}
