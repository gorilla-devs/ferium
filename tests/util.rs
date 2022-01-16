use std::process::Command;

pub fn run_command(args: Vec<&str>) -> std::io::Result<()> {
	match Command::new(env!("CARGO_BIN_EXE_ferium"))
		.args(args)
		.output()
	{
		Ok(out) => {
			if out.status.success() {
				Ok(())
			} else {
				Err(std::io::Error::new(
					std::io::ErrorKind::Other,
					format!(
						"Command returned with exit code {:?}.\nStdout dump:{}",
						out.status.code(),
						std::str::from_utf8(&out.stdout).unwrap(),
					),
				))
			}
		}
		Err(err) => Err(err),
	}
}
