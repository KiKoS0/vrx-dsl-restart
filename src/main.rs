use std::fs::OpenOptions;
use std::io::Write;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

//TODO: Tone it down to only get the last X lines or figure out how to stream from the file.
fn execute() -> bool {
    let mut cmd = Command::new("dmesg")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let restart_msg_flag = "Restarted DSL connection";
    let showtime_enter_flag = "vrx518_tc:ptm_showtime_enter";
    let showtime_exit_flag = "vrx518_tc:ptm_showtime_exit";

    let stdout = cmd.stdout.as_mut().unwrap();
    let stdout_reader = BufReader::new(stdout);
    let stdout_lines = stdout_reader.lines();

    let lines: Vec<String> = stdout_lines.map(|line| line.unwrap()).collect();
    for line in lines.iter().rev() {
        if line.contains(restart_msg_flag) || line.contains(showtime_enter_flag) {
            dbg_print_to_kernel_log("Skipping because of restart or showtime enter flag");
            return false;
        }

        if line.contains(showtime_exit_flag) {
            restart_dsl_connection();

            print_to_kernel_log(restart_msg_flag);

            dbg_print_to_kernel_log("Sleeping for 60 seconds");
            std::thread::sleep(std::time::Duration::from_secs(60));
            return true;
        }
    }

    cmd.wait().unwrap();

    false
}

fn restart_dsl_connection() {
    let cmds = vec![
        "/etc/init.d/dsl_control stop",
        "service tailscale stop",
        "sleep 5",
        "rmmod drv_dsl_cpe_api",
        "sleep 1",
        "rmmod drv_mei_cpe",
        "sleep 1",
        "rmmod vrx518_tc",
        "sleep 1",
        "rmmod vrx518",
        "sleep 1",
        "modprobe vrx518",
        "sleep 1",
        "modprobe vrx518_tc",
        "sleep 1",
        "modprobe drv_mei_cpe",
        "sleep 1",
        "modprobe drv_dsl_cpe_api",
        "sleep 1",
        "/etc/init.d/dsl_control start",
        "sleep 10",
        "service tailscale start",
    ];

    for cmd in cmds.iter() {
        let mut cmd = Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        let stdout = cmd.stdout.as_mut().unwrap();
        let stdout_reader = BufReader::new(stdout);
        let stdout_lines = stdout_reader.lines();

        for line in stdout_lines {
            dbg_print_to_kernel_log(line.unwrap().as_str());
        }

        cmd.wait().unwrap();
    }
}

fn main() {
    loop {
        if execute() {
            continue;
        }

        dbg_print_to_kernel_log("Sleeping for 10 seconds");
        std::thread::sleep(std::time::Duration::from_secs(10));
    }
}

fn dbg_print_to_kernel_log(msg: &str) {
    do_print_to_kernel_log(msg, true)
}

fn print_to_kernel_log(msg: &str) {
    do_print_to_kernel_log(msg, false)
}

fn do_print_to_kernel_log(msg: &str, debug: bool) {
    if debug && !cfg!(debug_assertions) {
        return;
    }
    let msg: String = format!("{}: {}", env!("CARGO_PKG_NAME"), msg);
    println!("{}", msg);
    let mut file = match OpenOptions::new().write(true).open("/dev/kmsg") {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Failed to open /dev/kmsg: {}", e);
            return;
        }
    };

    if let Err(e) = write!(file, "{}", msg) {
        eprintln!("Failed to write to /dev/kmsg: {}", e);
    }
}
