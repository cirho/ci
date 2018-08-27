#[macro_use]
extern crate structopt;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate failure_derive;
#[macro_use]
extern crate serde_derive;
extern crate chrono;
extern crate codeforces;
extern crate colored;
extern crate itertools;
extern crate keyring;
extern crate pbr;
extern crate reqwest;
extern crate rpassword;
extern crate select;
extern crate serde;
extern crate serde_json;
extern crate sio2;
extern crate tar;
extern crate tempfile;
extern crate wait_timeout;
extern crate walkdir;

mod auth;
mod checkers;
mod diagnose;
mod error;
mod strres;
mod testing;
#[macro_use]
mod ui;
mod cli;
mod commands;
mod fitness;
mod util;

use cli::{Args, Command};
use colored::Colorize;
use commands::build::Codegen;
use error::*;
use std::borrow::Borrow;
use structopt::StructOpt;

fn run() -> R<()> {
	let args = Args::from_args();
	let Args { ui, command } = args;
	match command {
		Command::Build {
			source,
			release,
			profile,
			standard,
			library,
		} => {
			let codegen = match (release, profile) {
				(false, false) => Codegen::Debug,
				(true, false) => Codegen::Release,
				(false, true) => Codegen::Profile,
				(true, true) => return Err(format_err!("both --release and --profile specified")),
			};
			commands::build::run(source.as_path(), &codegen, &standard, library.as_ref().map(|p| p.as_path()))
		},
		Command::Test {
			executable,
			testdir,
			checker,
			no_print_success,
			print_output,
		} => commands::test::run(executable.as_path(), testdir.as_path(), checker.borrow(), no_print_success, print_output, ui.borrow()),
		Command::Multitest {
			gen,
			executables,
			checker,
			count,
			fitness,
			time_limit,
			ignore_generator_fail,
		} => commands::multitest::run(
			gen.as_path(),
			&executables,
			checker.as_ref(),
			count,
			fitness.borrow(),
			time_limit,
			ignore_generator_fail,
			ui.borrow(),
		),
		Command::Vendor { source } => commands::vendor::run(source.as_path()),
		Command::GenerateAutocomplete { shell } => commands::genautocomplete::run(shell),
		Command::Init { url } => commands::init::run(&url, ui.borrow()),
		Command::Submit { source, url } => commands::submit::run(&url, &source, ui.borrow()),
		Command::TrackSubmit { url, id, sleep_duration } => commands::tracksubmit::run(&url, id, sleep_duration, ui.borrow()),
	}
}

fn main() {
	if let Err(e) = run() {
		let error_prefix = "error".red().bold();
		let cause_prefix = "caused by".yellow().bold();
		eprintln!("{}: {}", error_prefix, e);
		for cause in e.iter_causes().skip(1) {
			eprintln!("{}: {}", cause_prefix, cause);
		}
		::std::process::exit(1);
	}
}
