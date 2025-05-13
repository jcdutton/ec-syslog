use syslog::{Facility, Formatter3164};
use daemonize::Daemonize;
use std::{
    fs::File,
    fs::OpenOptions,
    io::{BufRead, BufReader},
    process,
    thread,
    time::Duration,
};

fn main() {
    // Daemonization setup
    //let stdout = File::create("/tmp/ec-syslog.out").unwrap();
    //let stderr = File::create("/tmp/ec-syslog.err").unwrap();

    // Open the file in append mode, create it if it doesn't exist.
    // Get the current process ID now that we're running as a daemon.
    let current_pid = process::id();
    // Initialize syslog
    let formatter = Formatter3164 {
        facility: Facility::LOG_DAEMON,
        hostname: None,
        process: "ec-syslog".into(),
        pid: current_pid as i32, // Insert the current PID.0,
    };

    let mut logger = syslog::unix(formatter).expect("Could not connect to syslog");

    // Define the path to the file to tail.
    let file_path = "/sys/kernel/debug/cros_ec/console_log";

    // Open the file for reading.
    //let file = File::open(file_path).expect("Could not open input file");
    let file = match File::open(file_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Could not open input file: {}: {} Suggest: modprobe cros_ec_debugfs", e, file_path);
            logger
                .info( format!("Could not open input file: {}: {} Suggest: modprobe cros_ec_debugfs", e, file_path) )
                .expect("Failed to write to syslog");
            return;
        }
    };

    let stdout = OpenOptions::new()
        .append(true)  // Open in append mode.
        .create(true)  // Create file if it doesn't exist.
        .open("/tmp/ec-syslog.out").unwrap();

    let stderr = OpenOptions::new()
        .append(true)  // Open in append mode.
        .create(true)  // Create file if it doesn't exist.
        .open("/tmp/ec-syslog.err").unwrap();

    let daemonize = Daemonize::new()
        .pid_file("/tmp/ec-syslog.pid") // Path to pid file
        .chown_pid_file(true) // Change ownership of the pid file
        .working_directory("/") // Set working directory
        .stdout(stdout) // Redirect stdout
        .stderr(stderr); // Redirect stderr

    match daemonize.start() {
        Ok(_) => println!("Daemonized successfully."),
        Err(e) => {
            eprintln!("Error daemonizing: {}: Suggestion: Perhaps ec-syslog is already running", e);
            return;
        }
    }

    let mut reader = BufReader::new(file);

    // Continuously poll for new lines.
    loop {
        let mut line = String::new();
        let result = reader.read_line(&mut line);
        //let mut line2: String  ="".to_string();
        let line2: String;
        match result {
            Ok(0) => {
                // No new input; sleep briefly before trying again.
                thread::sleep(Duration::from_secs(1));
            }
            Ok(_) => {
                // Trim the newline from the line and log it along with the process ID.
                line2 = line.clone().trim_end().to_string();
                let trimmed_line = line2;
                logger
                    .info( trimmed_line )
                    .expect("Failed to write to syslog");
            }
            Err(e) => {
                logger
                    .err(format!("Error reading line: {}", e))
                    .expect("Failed to write to syslog");
                // Sleep briefly on error before retrying.
                thread::sleep(Duration::from_secs(1));
            }
        }
    }
}

