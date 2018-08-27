use checkers::Checker;
use colored::Colorize;
use diagnose::*;
use error::*;
use fitness::Fitness;
use std::{
	borrow::Borrow, path::{Path, PathBuf}, time::Duration
};
use strres::{exec, StrRes};
use testing::{test_single, TestResult};
use ui::{timefmt, Ui};
use util::timefn;

pub fn run(
	gen: &Path,
	executables: &[PathBuf],
	checker: &Checker,
	count: Option<i64>,
	fitness: &Fitness,
	time_limit: Option<Duration>,
	ignore_generator_fail: bool,
	_ui: &Ui,
) -> R<()> {
	for executable in executables.iter() {
		diagnose_app(executable)?;
	}
	diagnose_checker(checker)?;
	let mut i = 1;
	let mut best: Option<(StrRes, i64)> = None;
	while count.map(|n| i <= n).unwrap_or(true) {
		let test_str = match exec(&gen, StrRes::Empty, None) {
			Ok(test_str) => test_str,
			e => if !ignore_generator_fail {
				e.context("failed to run test generator")?;
				StrRes::Empty
			} else {
				eprint_flush!("{}", format!("(gen fail      {:>6}", i).yellow().bold());
				continue;
			},
		};
		eprint_flush!("(autogenerated {:>6})", i);

		let (out1, t1) = timefn(|| exec(Path::new(&executables[0]), test_str.clone(), time_limit.as_ref()));

		let out1 = if let Ok(out1) = out1 {
			eprint_flush!(" {}", timefmt(t1).green().bold());
			Some(out1)
		} else {
			eprint_flush!(" {}", timefmt(t1).red().bold());
			None
		};

		let mut all_succeded = out1.is_some();
		for execi in executables[1..].iter() {
			if let Some(perfout) = out1.as_ref() {
				let (_out, outi, ti) = test_single(execi, test_str.clone(), perfout.clone(), checker.borrow(), time_limit.as_ref())?;
				let rawmsg = timefmt(ti);
				let msg = outi.apply_color(&rawmsg);
				eprint_flush!(" {}", msg);

				if outi != TestResult::Accept {
					all_succeded = false;
				}
			} else {
				eprint_flush!(" {}", "-.--s".yellow().bold());
			}
		}
		eprintln!();
		if !all_succeded {
			if count.is_none() {
				test_str.print_to_stdout();
				break;
			} else {
				let fit = fitness.fitness(test_str.clone())?;
				if best.as_ref().map(|&(_, bfit)| fit > bfit).unwrap_or(true) {
					best = Some((test_str, fit));
				}
			}
		}
		i += 1;
	}

	if count.is_some() {
		if let Some((test_str, _)) = best {
			test_str.print_to_stdout();
		}
	}

	Ok(())
}
