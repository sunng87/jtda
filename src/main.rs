#[macro_use] extern crate lazy_static;

extern crate regex;
extern crate clap;

mod dump;
mod analyze;

use std::path::Path;
use std::io;
use std::io::prelude::*;
use std::fs::File;
use clap::App;

fn load_from_io<'a>(i: Option<&'a str>) -> io::Result<String> {
    let mut s = String::new();
    if let Some(fname) = i {
        let mut f = try!(File::open(Path::new(fname)));
        try!(f.read_to_string(&mut s));
    } else {
        let mut f = io::stdin();
        try!(f.read_to_string(&mut s));
    };
    Ok(s)
}

fn main() {
    let matches = App::new("jtda")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Ning Sun <sunng@about.me>")
        .about("Java Thread Dump Analyzer")
        .args_from_usage(
            "[file]... 'The dump file, use stdin if not given'")
        .get_matches();

    match load_from_io(matches.value_of("file")) {
        Ok(s) => {
            let tda = dump::JThreadDump::from(s.as_ref());

            let result = analyze::by_state(&tda);
            for (k, v) in result.iter() {
                println!("{:?}", k);
                for t in v.iter() {
                    println!("\t\t{}", t.name);
                }
            }

            println!("=================================");

            let result2 = analyze::by_stacktrace(&tda);
            for (k, v) in result2.iter() {
                for t in v.iter() {
                    println!("{}", t.name);
                }

                println!("{}", k);
            }
        },
        Err(e) => {
            panic!(format!("Failed to load dump: {}", e))
        }
    };
}
