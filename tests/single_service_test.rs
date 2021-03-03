use control::synchronization::Synchronization;
use control::telemetry::{SelectGame, Telemetry};
use control::{ServiceType, Services};
use std::thread;
use std::time::{Duration, Instant};

mod emulation_start;

use emulation_start::ETS2Emulation;
use std::io::BufRead;

#[test]
fn telemetry_service_shutdown_when_completed() {
    //game selection
    let sel_game = SelectGame::ETS2;
    // setup telemetry
    let mut ets2_telemetry = Telemetry::via_shared_memory(sel_game);

    let sync = Synchronization::new();

    let tx = sync.get_transmitter();

    ets2_telemetry.set_transmitter(tx);

    let mut cap_sess = Services::new();

    let reader = ETS2Emulation::start_with_stdout();

    thread::sleep(Duration::from_secs(1));
    cap_sess
        .add_service(ServiceType::TelemetryInput, Box::new(ets2_telemetry))
        .unwrap();

    let mut i: u64 = 0;
    for line in reader.lines() {
        let x = line.expect("irrelevant error - may ignore");
        let numbers: Vec<u64> = x
            .split('-')
            .map(|val| str::parse::<u64>(val).unwrap())
            .collect();

        assert!(i == numbers[0]);
        i += 1;
    }

    assert!(i == 23182);

    cap_sess.block_until_telemetry_finished().unwrap();

    //cap_sess.stop_all_services();
}

#[test]
fn telemetry_service_shutdown_interrupt_all_services() {
    let sel_game = SelectGame::ETS2;
    // setup telemetry
    let mut ets2_telemetry = Telemetry::via_shared_memory(sel_game);

    let sync = Synchronization::new();

    let tx = sync.get_transmitter();

    ets2_telemetry.set_transmitter(tx);

    let mut cap_sess = Services::new();
    thread::sleep(Duration::from_secs(1));
    cap_sess
        .add_service(ServiceType::TelemetryInput, Box::new(ets2_telemetry))
        .unwrap();
    let t = Instant::now();
    thread::sleep(Duration::from_millis(50));
    cap_sess.stop_all_services().unwrap();

    let g = Instant::now().duration_since(t).as_millis();

    assert!(g < 100);
}

#[test]
fn telemetry_service_shutdown_interrupt_stop_service() {
    let sel_game = SelectGame::ETS2;
    // setup telemetry
    let mut ets2_telemetry = Telemetry::via_shared_memory(sel_game);

    let sync = Synchronization::new();

    let tx = sync.get_transmitter();

    ets2_telemetry.set_transmitter(tx);

    let mut cap_sess = Services::new();
    thread::sleep(Duration::from_secs(1));
    cap_sess
        .add_service(ServiceType::TelemetryInput, Box::new(ets2_telemetry))
        .unwrap();
    let t = Instant::now();
    thread::sleep(Duration::from_millis(50));
    cap_sess.stop_service(ServiceType::TelemetryInput).unwrap();

    let g = Instant::now().duration_since(t).as_millis();

    assert!(g < 100);
}
