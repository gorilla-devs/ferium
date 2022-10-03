use std::process::Command;

use std::fs::{copy, create_dir};
use std::io::Result;

pub fn run_command(args: Vec<&str>, config_file: Option<&str>) -> Result<()> {
    let mut args = args;
    let running = format!("./tests/configs/running/{}.json", rand::random::<u16>());
    if let Some(config_file) = config_file {
        let _ = create_dir("./tests/configs/running");
        let template = format!("./tests/configs/{}.json", config_file);
        copy(&template, &running)?;
    }

    let mut command = Command::new(env!("CARGO_BIN_EXE_ferium"));
    let mut arguments = Vec::new();
    arguments.push("--config-file");
    arguments.push(&running);
    arguments.append(&mut args);
    command.args(arguments);
    let output = command.output()?;
    if output.status.success() {
        Ok(())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!(
                "Command returned with exit code {:?}, stdout:{}, stderr:{}",
                output.status.code(),
                std::str::from_utf8(&output.stdout).unwrap(),
                std::str::from_utf8(&output.stderr).unwrap(),
            ),
        ))
    }
}
