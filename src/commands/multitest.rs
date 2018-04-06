use cli::Args;
use diagnose::diagnose_app;
use strres::{exec, StrRes};
use testing::{test_single, TestResult};
use ui::timefmt;
use util::timefn;
use std::path::Path;
use colored::*;

pub fn run(args: Args) {
	if let Args::Multitest { gen, executables, checker } = args {
		for ref executable in &executables {
			diagnose_app(executable);
		}
		let mut i = 1;
		loop {
			let test_str = exec(&gen, StrRes::Empty).unwrap();
			print_flush!("(autogenerated {:<6})", i);

			let (out1, t1) = timefn(|| exec(Path::new(&executables[0]), test_str.clone()));
			let out1 = if let Ok(out1) = out1 {
				print_flush!(" {}", timefmt(t1).green().bold());
				out1
			} else {
				print_flush!(" {}", timefmt(t1).red().bold());
				test_str.print_to_stdout();
				break
			};

			let mut all_succeded = true;
			for ref execi in &executables[1..] {
				let (outi, ti) = test_single(execi, test_str.clone(), out1.clone(), checker.as_ref());
				let msg = outi.apply_color(&timefmt(ti));
				print_flush!(" {}", msg);

				if outi != TestResult::Accept {
					all_succeded = false;
				}
			}
			println!("");
			if !all_succeded {
				test_str.print_to_stdout();
				break
			}
			i += 1;
		}
	}
}