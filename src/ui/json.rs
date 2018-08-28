use super::Ui;
use commands::{self, tracksubmit::Examples};
use serde_json;
use std::{
	io::{stdin, stdout, Write}, path::Path, time::Duration
};
use strres::StrRes;
use testing::TestResult;
use ui::ProgressBar;

pub struct Json;
impl Json {
	pub fn new() -> Json {
		Json
	}
}
impl Ui for Json {
	fn read_auth(&self, domain: &str) -> (String, String) {
		println!("{}", serde_json::to_string(&AuthRequest { domain: domain.to_owned() }).unwrap());
		stdout().flush().unwrap();
		let mut line = String::new();
		stdin().read_line(&mut line).unwrap();
		let resp: AuthResponse = serde_json::from_str(&line).unwrap();
		(resp.username, resp.password)
	}

	fn create_progress_bar(&self, _: usize) -> Box<ProgressBar> {
		Box::new(NoProgressBar)
	}

	fn track_progress(&self, status: &commands::tracksubmit::Status) {
		fn convert_examples(examples: &Examples) -> bool {
			match examples {
				Examples::OK => true,
				Examples::Wrong => false,
			}
		}
		let obj = match status {
			commands::tracksubmit::Status::InitialPending => TrackProgress {
				status: TrackStatus::InitialPending,
				examples_succeded: None,
				score: None,
			},
			commands::tracksubmit::Status::ScorePending { examples } => TrackProgress {
				status: TrackStatus::ScorePending,
				examples_succeded: Some(convert_examples(examples)),
				score: None,
			},
			commands::tracksubmit::Status::ScoreReady { examples, score } => TrackProgress {
				status: TrackStatus::ScoreReady,
				examples_succeded: Some(convert_examples(examples)),
				score: Some(*score),
			},
		};
		println!("{}", serde_json::to_string(&obj).unwrap());
	}

	fn submit_success(&self, id: String) {
		println!("{}", serde_json::to_string(&SubmitResult { id }).unwrap());
	}
}

struct NoProgressBar;
impl ProgressBar for NoProgressBar {
	fn print_test(&mut self, outcome: &TestResult, timing: Option<Duration>, in_path: &Path, output: Option<StrRes>) {
		println!(
			"{}",
			serde_json::to_string(&SingleTest {
				outcome,
				timing,
				in_path,
				output: output.map(|sr| sr.get_string().unwrap())
			}).unwrap()
		);
	}

	fn increment(&mut self) {}
}

#[derive(Serialize)]
struct AuthRequest {
	domain: String,
}
#[derive(Deserialize)]
struct AuthResponse {
	username: String,
	password: String,
}
#[derive(Serialize)]
struct SingleTest<'a> {
	outcome: &'a TestResult,
	timing: Option<Duration>,
	in_path: &'a Path,
	output: Option<String>,
}

#[derive(Serialize)]
struct SubmitResult {
	id: String,
}

#[derive(Serialize)]
enum TrackStatus {
	InitialPending,
	ScorePending,
	ScoreReady,
}
#[derive(Serialize)]
struct TrackProgress {
	status: TrackStatus,
	examples_succeded: Option<bool>,
	score: Option<i64>,
}