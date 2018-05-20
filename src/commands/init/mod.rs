mod codeforces;
mod oioioi;

use sio2::Url;
use util::{writefile, demand_dir};
use error::*;

pub struct Test {
	input: String,
	output: String,
}

pub trait Site {
	fn download_tests(url: &Url) -> Vec<Test>;
}

pub fn run(url: Url) -> R<()> {
	demand_dir("./tests/").context("failed to create tests directory")?;
	demand_dir("./tests/example/").context("failed to create tests directory")?;
	let tests = acquire_tests(&url).context("failed to downloade tests")?;
	for (i, test) in tests.into_iter().enumerate() {
		writefile(&format!("./tests/example/{}.in", i+1), &test.input);
		writefile(&format!("./tests/example/{}.out", i+1), &test.output);
	}
	Ok(())
}

const MATCHERS: &[(&'static str, fn(&Url) -> Vec<Test>)] = &[
	("codeforces.com", codeforces::Codeforces::download_tests),
	("sio2.staszic.waw.pl", oioioi::Oioioi::download_tests),
];

fn acquire_tests(url: &Url) -> R<Vec<Test>> {
	let domain = url.domain().unwrap();
	MATCHERS.iter().find(|&&(dom, _)| dom == domain)
		.ok_or(Error::from(E::UnsupportedProblemSite(domain.to_owned())))
		.map(move |(_, f)| f(url))
}