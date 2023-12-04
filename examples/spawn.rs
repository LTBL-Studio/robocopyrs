// An example to show how to spawn the underlying std::process::Command and read its stdout
use std::{path::Path, io::{BufReader, BufRead}, process::{Command, Stdio}};

use robocopyrs::{
    RobocopyCommandBuilder, logging::LoggingOptions, exit_codes::OkExitCode,
};

fn main() {
    let command = RobocopyCommandBuilder {
        source: Path::new("."),
        destination: Path::new("./copy"),
        files: vec!["*"],
        only_copy_top_n_levels: Some(1),
        logging: Some(LoggingOptions {
            dont_log_class: true,
            dont_log_dir_names: true,
            show_estimated_time_of_arrival: true,
            dont_log_header: true,
            dont_log_summary: true,
            ..Default::default()
        }),
        ..Default::default()
    }
    .build();

    println!("Built command is : {command:?}");
    
    let mut command: Command = command.into();
    let mut process = command.stdout(Stdio::piped()).spawn().expect("Error during command spawning");
    let stdout = process.stdout.take().unwrap();

    let mut buf = Vec::new();
    let mut stdout_reader = BufReader::new(stdout);

    while let Ok(stdout_lines) = stdout_reader.read_until(b'\r', &mut buf) { // Prints the progress
        if stdout_lines == 0 {
            break;
        }
        println!("Read: {:?}", String::from_utf8_lossy(&buf));
        buf.clear();
    }

    let exit_code = process.wait().expect("Command wasn't running").code().expect("Process terminated by signal") as i8;

    match OkExitCode::try_from(exit_code) {
        Ok(success_code) => println!("Copy was successful: {success_code:?}"),
        Err(err_code) => eprintln!("Copy encoutered some errors: {err_code:?}"),
    };
}
