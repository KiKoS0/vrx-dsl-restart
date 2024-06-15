use std::fs::OpenOptions;
use std::io::Write;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::time::{self, Duration};

//TODO: Tone it down to only get the last X lines or figure out how to stream from the file.
async fn execute() -> bool {
    let mut cmd = Command::new("dmesg")
        .stdout(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .unwrap();

    let restart_msg_flag = "Restarted DSL connection";
    let showtime_enter_flag = "vrx518_tc:ptm_showtime_enter";
    let showtime_exit_flag = "vrx518_tc:ptm_showtime_exit";

    let stdout = cmd.stdout.as_mut().unwrap();
    let mut stdout_reader = BufReader::new(stdout).lines();

    let mut lines: Vec<String> = Vec::new();
    while let Some(line) = stdout_reader.next_line().await.unwrap() {
        lines.push(line);
    }

    for line in lines.iter().rev() {
        if line.contains(restart_msg_flag) || line.contains(showtime_enter_flag) {
            dbg_print_to_kernel_log("Skipping because of restart or showtime enter flag");
            return false;
        }

        if line.contains(showtime_exit_flag) {
            restart_dsl_connection().await;
            print_to_kernel_log(restart_msg_flag);
            dbg_print_to_kernel_log("Sleeping for 60 seconds");

            time::sleep(Duration::from_secs(60)).await;
            return true;
        }
    }

    cmd.wait().await.unwrap();

    false
}

async fn restart_dsl_connection() {
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
        let mut child = Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .stdout(std::process::Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .unwrap();

        if let Some(stdout) = child.stdout.take() {
            let mut stdout_reader = BufReader::new(stdout).lines();

            while let Ok(Some(line)) = stdout_reader.next_line().await {
                dbg_print_to_kernel_log(&line);
            }
        }

        child.wait().await.unwrap();
    }
}

#[tokio::main]
async fn main() {
    loop {
        if execute().await {
            continue;
        }

        dbg_print_to_kernel_log("Sleeping for 10 second");
        time::sleep(Duration::from_secs(10)).await;
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
