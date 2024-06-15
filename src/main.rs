use tokio::fs::File;
use tokio::io::{self, AsyncBufReadExt, AsyncSeekExt, BufReader, SeekFrom};
use tokio::process::Command;
use tokio::time::{self, Duration};

#[tokio::main]
async fn main() {
    loop {
        process_kmsg().await.unwrap();
    }
}

async fn process_kmsg() -> io::Result<()> {
    let mut file = File::open("/dev/kmsg").await?;
    file.seek(SeekFrom::End(0)).await?;

    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    let showtime_exit_flag = "vrx518_tc:ptm_showtime_exit";

    dbg_print("Listening...");
    while let Some(line) = lines.next_line().await? {
        if line.contains(showtime_exit_flag) {
            restart_dsl_connection().await;

            dbg_print("Sleeping for 15 seconds, avoiding restart loop.");
            time::sleep(Duration::from_secs(15)).await;

            return Ok(());
        }
    }

    Ok(())
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
                dbg_print(&line);
            }
        }

        child.wait().await.unwrap();
    }
}

fn dbg_print(msg: &str) {
    println!("{}: {}", env!("CARGO_PKG_NAME"), msg);
}
