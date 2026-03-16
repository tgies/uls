//! Shell completion generation.

use std::io;

use clap::CommandFactory;
use clap_complete::{generate, Shell};

use crate::Cli;

pub fn execute(shell: Shell) {
    let mut cmd = Cli::command();
    generate(shell, &mut cmd, "uls", &mut io::stdout());
}
