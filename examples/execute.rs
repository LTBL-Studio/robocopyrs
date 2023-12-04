use std::path::Path;

use robocopyrs::RobocopyCommandBuilder;

fn main() {
    let mut command = RobocopyCommandBuilder {
        source: Path::new("."),
        destination: Path::new("./copy"),
        files: vec!["*"],
        only_copy_top_n_levels: Some(1),
        ..Default::default()
    }
    .build();

    println!("Built command is : {command:?}");

    match command.execute() {
        Ok(code) => {
            println!("{code:?}")
        }
        Err(err) => {
            eprintln!("Exit code error: {err:?}")
        }
    };
}
