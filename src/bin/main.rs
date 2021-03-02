use control::telemetry::{SelectGame, Telemetry};
use control::{ServiceType, Services};
use std::io::{BufRead, BufReader, Error, ErrorKind};
use std::process::{Command, Stdio};
use std::rc::Rc;
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

    //start child process for telemetry emulation
    // let child = Command::new(".\\tests.\\TelemetryEmulation_ets2_hard_data_ref.exe")
    //     .stdout(Stdio::piped())
    //     .spawn()
    //     .expect("failed to execute child");

    let stdout = Command::new(".\\tests.\\TelemetryEmulation_ets2_hard_data_ref.exe")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap()
        .stdout
        .unwrap();

    thread::sleep(Duration::from_secs(1));
    cap_sess
        .add_service(ServiceType::telemetry_input, Box::new(ets2_telemetry))
        .unwrap();

    let reader = BufReader::new(stdout);

    reader
        .lines()
        .for_each(|line| println!("server send: {}", line.ok().unwrap()));

    cap_sess.block_until_telemetry_finished().unwrap();
    println!("telemetry service stopped");

    //let output = child.wait_with_output().expect("failed to wait for child");

    // String::from_utf8(output.stdout)
    //     .unwrap()
    //     .lines()
    //     .take(5)
    //     .for_each(|x| println!("{}", x));

    // kill child process
    //child.kill().unwrap();
}
