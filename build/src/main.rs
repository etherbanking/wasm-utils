//! Experimental build tool for cargo

extern crate glob;
extern crate wasm_utils;
extern crate clap;
extern crate parity_wasm;

use std::{fs, io};
use std::path::PathBuf;

use clap::{App, Arg};

#[derive(Debug)]
pub enum Error {
	Io(io::Error),
	NoSuitableFile(String),
	TooManyFiles(String),
	NoEnvVar,
}

impl From<io::Error> for Error {
	fn from(err: io::Error) -> Self {
		Error::Io(err)
	}
}

pub fn wasm_path(target_dir: &str, bin_name: &str) -> String {
	let mut path = PathBuf::from(target_dir);
	path.push(format!("{}.wasm", bin_name));
	path.to_string_lossy().to_string()
}

pub fn process_output(target_dir: &str, bin_name: &str) -> Result<(), Error> {
	let mut path = PathBuf::from(target_dir);
	let wasm_name = bin_name.to_string().replace("-", "_");
	path.push("wasm32-unknown-emscripten");
	path.push("release");
	path.push("deps");
	path.push(format!("{}-*.wasm", wasm_name));

	let mut files = glob::glob(path.to_string_lossy().as_ref()).expect("glob err")
		.collect::<Vec<Result<PathBuf, glob::GlobError>>>();

	if files.len() == 0 {
		return Err(Error::NoSuitableFile(path.to_string_lossy().to_string()));
	} else if files.len() > 1 {
		return Err(Error::TooManyFiles(
			files.into_iter().map(|f| f.expect("glob err").to_string_lossy().to_string())
				.fold(String::new(), |mut a, b| { a.push_str(", "); a.push_str(&b); a })
		))
	} else {
		let file = files.drain(..).nth(0).expect("0th element exists").expect("glob err");
		let mut path = PathBuf::from(target_dir);
		path.push(format!("{}.wasm", bin_name));
		fs::copy(file, path)?;
	}

	Ok(())
}

fn main() {
	wasm_utils::init_log();

	let matches = App::new("wasm-opt")
		.arg(Arg::with_name("target")
			.index(1)
			.required(true)
			.help("Cargo target directory"))
		.arg(Arg::with_name("wasm")
			.index(2)
			.required(true)
			.help("Wasm binary name"))
		.arg(Arg::with_name("skip_optimization")
			.help("Skip symbol optimization step producing final wasm")
			.long("skip-optimization"))
		.arg(Arg::with_name("skip_alloc")
			.help("Skip allocator externalizer step producing final wasm")
			.long("skip-externalize"))
		.arg(Arg::with_name("runtime_type")
			.help("Injects RUNTIME_TYPE global export")
			.takes_value(true)
			.long("runtime-type"))
		.arg(Arg::with_name("runtime_version")
			.help("Injects RUNTIME_VERSION global export")
			.takes_value(true)
			.long("runtime-version"))
		.get_matches();

    let target_dir = matches.value_of("target").expect("is required; qed");
    let wasm_binary = matches.value_of("wasm").expect("is required; qed");

	process_output(target_dir, wasm_binary).expect("Failed to process cargo target directory");

	let path = wasm_path(target_dir, wasm_binary);

	let mut module = parity_wasm::deserialize_file(&path).unwrap();

	if !matches.is_present("skip_alloc") {
		module = wasm_utils::externalize(
			module,
			vec!["_free", "_malloc"],
		);
	}

	if !matches.is_present("skip_optimization") {
		wasm_utils::optimize(&mut module, vec!["_call", "setTempRet0"]).expect("Optimizer to finish without errors");
	}

	if let Some(runtime_type) = matches.value_of("runtime_type") {
		let runtime_type: &[u8] = runtime_type.as_bytes();
		if runtime_type.len() != 4 {
			panic!("--runtime-type should be equal to 4 bytes");
		}
		let runtime_version: u32 = matches.value_of("runtime_version").unwrap_or("1").parse()
			.expect("--runtime-version should be a positive integer");
		module = wasm_utils::inject_runtime_type(module, &runtime_type, runtime_version);
	}

	parity_wasm::serialize_to_file(&path, module).unwrap();
}
