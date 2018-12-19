use super::Ui;
use chrono::Local;
use commands::{
	list_resources::Resource, tracksubmit::{Compilation, Outcome, Status}
};
use failure;
use rpassword;
use std::{
	self, io::{stderr, stdin, stdout, Write}, path::Path, time::Duration
};
use strres::StrRes;
use term;
use testing::TestResult;
use ui::timefmt;

pub struct Human {
	term: Box<term::StderrTerminal>,
}
impl Human {
	pub fn new() -> Human {
		Human {
			term: term::stderr().expect("Failed to obtain terminal"),
		}
	}

	fn bcol(&mut self, color: term::color::Color, f: impl FnOnce()) {
		self.term.fg(color).unwrap();
		self.term.attr(term::Attr::Bold).unwrap();
		f();
		self.term.reset().unwrap();
	}
}
impl Ui for Human {
	fn read_auth(&mut self, domain: &str) -> (String, String) {
		eprintln!("Login required to {}", domain);
		eprint!("  Username: ");
		stderr().flush().unwrap();
		let mut username = String::new();
		stdin().read_line(&mut username).unwrap();
		username = username.trim().to_owned();
		let password = rpassword::prompt_password_stderr("  Password: ").unwrap();
		(username, password)
	}

	fn track_progress(&mut self, status: &Status) {
		let message = match status {
			Status {
				compilation: Compilation::Pending,
				..
			} => "Compilation pending...".to_string(),
			Status {
				compilation: Compilation::Failure,
				..
			} => "Compilation error".to_string(),
			Status { initial: Outcome::Pending, .. } => "Initial tests pending...".to_string(),
			Status {
				full: Outcome::Pending, initial, ..
			} => format!("{}Score pending...", self.fmt_initial(initial)),
			Status { full, .. } => self.fmt_full(full),
		};
		eprintln!("{} {}", Local::now(), message);
	}

	fn submit_success(&mut self, id: String) {
		eprintln!("Solution submitted, submission id: {}", id);
	}

	fn print_resource_list(&mut self, resources: &[Resource]) {
		for resource in resources {
			self.bcol(term::color::WHITE, || eprintln!("{}", resource.name));
			eprintln!("{}", resource.description);
			eprintln!("{}", resource.filename);
			eprintln!("{}", resource.id);
			eprintln!();
		}
	}

	fn print_resource(&mut self, data: &'_ [u8]) {
		stdout().write_all(&data).unwrap();
		stdout().flush().unwrap();
	}

	fn print_test(&mut self, outcome: &TestResult, timing: Option<Duration>, in_path: &Path, output: Option<StrRes>) {
		let rstr = self.fmt_test_long(outcome);
		let timestr = timing.map(timefmt).unwrap_or("-.--s".to_owned());
		self.bcol(color_testr(outcome), || eprint!("{} ", rstr));
		self.bcol(term::color::BLUE, || eprint!("{} ", timestr));
		eprintln!("{}", in_path.display());
		if let Some(output) = output {
			eprintln!("{}", output.get_string().unwrap());
		}
	}

	fn print_finish_test(&mut self, _success: bool) {}

	fn print_transpiled(&mut self, compiled: &str) {
		println!("{}", compiled);
	}

	fn print_found_test(&mut self, test_str: &str) {
		println!("{}", test_str);
	}

	fn print_error(&mut self, error: failure::Error) {
		self.bcol(term::color::RED, || eprint!("error"));
		eprintln!(": {}", error);
		for cause in error.iter_causes().skip(1) {
			self.bcol(term::color::YELLOW, || eprint!("caused by"));
			eprintln!(": {}", cause);
		}
	}

	fn mt_generator_fail(&mut self, i: i64) {
		self.bcol(term::color::YELLOW, || eprint_flush!("(gen fail      {:>6}", i));
	}

	fn mt_autogenerated(&mut self, i: i64) {
		eprint_flush!("(autogenerated {:>6})", i);
	}

	fn mt_good(&mut self, t: Duration) {
		self.bcol(term::color::GREEN, || eprint_flush!(" {}", timefmt(t)));
	}

	fn mt_bad(&mut self, t: Duration) {
		self.bcol(term::color::RED, || eprint_flush!(" {}", timefmt(t)));
	}

	fn mt_piece(&mut self, outi: &TestResult, ti: Duration) {
		self.bcol(color_testr(outi), || eprint_flush!(" {}", timefmt(ti)));
	}

	fn mt_piece_ignored(&mut self) {
		self.bcol(term::color::YELLOW, || eprint_flush!(" -.--s"));
	}

	fn mt_piece_finish(&mut self) {
		eprintln!();
	}

	fn warn(&mut self, message: &str) {
		self.bcol(term::color::RED, || eprint!("WARNING"));
		eprint!(" {}. Continue? ", message);
		std::io::stderr().flush().unwrap();
		std::io::stdin().read_line(&mut String::new()).unwrap();
	}

	fn notice(&mut self, message: &str) {
		eprintln!("{}", message);
	}
}

impl Human {
	fn fmt_initial(&mut self, outcome: &Outcome) -> String {
		match outcome {
			Outcome::Score(score) => format!("Initial tests scored {}. ", score),
			Outcome::Failure => "Initial tests failed. ".to_string(),
			Outcome::Success => "Initial tests passed. ".to_string(),
			Outcome::Pending => panic!("formatting pending initiial tests"),
			Outcome::Unsupported => "".to_string(),
			Outcome::Skipped => "".to_string(),
			Outcome::Waiting => panic!("formatting waiting initial tests"),
		}
	}

	fn fmt_full(&mut self, outcome: &Outcome) -> String {
		match outcome {
			Outcome::Score(score) => format!("Scored {}!", score),
			Outcome::Failure => "Rejected".to_string(),
			Outcome::Success => "Accepted".to_string(),
			Outcome::Pending => panic!("formatting pending tests"),
			Outcome::Unsupported => panic!("formatting unsupported tests"),
			Outcome::Skipped => panic!("formatting skipped tests"),
			Outcome::Waiting => panic!("formatting waiting tests"),
		}
	}

	fn fmt_test_long(&mut self, outcome: &TestResult) -> &'static str {
		match outcome {
			TestResult::Accept => "ACCEPT       ",
			TestResult::WrongAnswer => "WRONG ANSWER ",
			TestResult::RuntimeError => "RUNTIME ERROR",
			TestResult::IgnoredNoOut => "IGNORED      ",
		}
	}
}

fn color_testr(outi: &TestResult) -> term::color::Color {
	match outi {
		TestResult::Accept => term::color::GREEN,
		TestResult::WrongAnswer => term::color::RED,
		TestResult::RuntimeError => term::color::RED,
		TestResult::IgnoredNoOut => term::color::YELLOW,
	}
}
