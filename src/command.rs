use std::ffi::OsStr;
use std::io::ErrorKind;
use std::process::{Command, ExitStatus, Output};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BetterCommandError {
    #[error("Command not found: {executable:?}")]
    NotFound { executable: String },

    #[error("{executable:?} failed with {status}\n{error:?}")]
    ExitStatusError {
        executable: String,
        status: ExitStatus,
        error: String,
    },

    #[error(transparent)]
    IOError { source: std::io::Error },
}

pub struct BetterCommand {
    executable: String,
    command: Command,
}

fn u8vec_to_string(s: Vec<u8>) -> String {
    String::from_utf8_lossy(&s).to_string()
}

impl BetterCommand {
    pub fn new(executable: &str) -> BetterCommand {
        BetterCommand {
            executable: executable.to_string(),
            command: Command::new(executable),
        }
    }

    pub fn arg<S: AsRef<OsStr>>(&mut self, arg: S) -> &mut BetterCommand {
        self.command.arg(arg.as_ref());
        self
    }

    pub fn args<I, S>(&mut self, args: I) -> &mut BetterCommand
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.command.args(args);
        self
    }

    pub fn output(&mut self) -> Result<Output, BetterCommandError> {
        self.command.output().map_err(|err| match err.kind() {
            ErrorKind::NotFound => BetterCommandError::NotFound {
                executable: self.executable.clone(),
            },
            _ => BetterCommandError::IOError { source: err },
        })
    }

    pub fn check_status_and_get_output_string(&mut self) -> Result<String, BetterCommandError> {
        let output = self.output()?;

        match output.status.success() {
            true => Ok(u8vec_to_string(output.stdout)),
            false => Err(BetterCommandError::ExitStatusError {
                executable: self.executable.clone(),
                status: output.status,
                error: u8vec_to_string(output.stderr),
            }),
        }
    }
}
