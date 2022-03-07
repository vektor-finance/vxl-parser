use std::{
  io::{self, Read},
  time::Instant,
};

use nom_tracable::{self, cumulative_histogram, histogram};
use serde_json::to_string_pretty;

use clap::{crate_version, App, Arg};

use core::parse;

fn main() {
  let matches = App::new("VXL")
    .version(crate_version!())
    .author("Vektor Engineering <engineering@vektor.finance>")
    .about("VXL parser")
    .arg(Arg::new("input").help("string to be parsed"))
    .arg(
      Arg::new("vxl-version")
        .short('x')
        .long("vxl-version")
        .help("print vxl parser version information"),
    )
    .arg(
      Arg::new("profile")
        .short('p')
        .long("profile")
        .help("print time profiling information"),
    )
    .get_matches();

  if matches.is_present("vxl-version") {
    println!(
      "VXL Version: {} ({})",
      env!("VERGEN_GIT_SHA_SHORT"),
      env!("VERGEN_BUILD_TIMESTAMP")
    );
    return;
  }

  let input = match matches.value_of("input") {
    Some(i) => i.to_string(),
    None => {
      let mut i = String::new();
      let mut stdin = io::stdin();
      stdin.read_to_string(&mut i).unwrap();
      i
    }
  };

  let now = Instant::now();
  let result = parse(&input).unwrap_or_else(|error| {
    panic!("Parse error: {}", error);
  });

  let parse_duration = now.elapsed();

  let now = Instant::now();
  let json = to_string_pretty(&result).unwrap_or_else(|error| {
    panic!("JSON serialisation error: {}", error);
  });
  let json_duration = now.elapsed();

  println!("{}", json);

  if matches.is_present("profile") {
    println!(
      "
Profiling:
----------
Parse duration: {:?}
JSON serialisation duration: {:?}
    ",
      parse_duration, json_duration
    );
  }

  if cfg!(feature = "trace") {
    println!("Trace information:");
    histogram();
    cumulative_histogram();
  }
}
