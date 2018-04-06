use checkers::Checker;
use strres::{exec, StrRes};
use util::timefn;
use std;
use std::path::Path;
use colored::*;

#[derive(PartialEq, Eq)]
pub enum TestResult {
	Accept,
	WrongAnswer,
	RuntimeError,
}
impl TestResult {
	pub fn format_long(&self) -> ColoredString {
		self.apply_color(match self {
			&TestResult::Accept =>       "ACCEPT       ",
			&TestResult::WrongAnswer =>  "WRONG ANSWER ",
			&TestResult::RuntimeError => "RUNTIME ERROR",

		})
	}
	pub fn apply_color(&self, s: &str) -> ColoredString {
		match self {
			&TestResult::Accept => s.green().bold(),
			&TestResult::WrongAnswer => s.red().bold(),
			&TestResult::RuntimeError => s.red().bold(),
		}
	}
}
pub fn test_single(executable: &Path, input: StrRes, perfect_output: StrRes, checker: &Checker) -> (TestResult, std::time::Duration) {
	let (my_output, timing) = timefn(|| exec(executable, input.clone()));
	(if let Ok(my_output) = my_output {
		if checker.check(input, my_output, perfect_output) {
			TestResult::Accept
		} else {
			TestResult::WrongAnswer
		}
	} else {
		TestResult::RuntimeError
	}, timing)
}