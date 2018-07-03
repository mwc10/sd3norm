#[macro_use] extern crate structopt;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate failure;
extern crate serde;
extern crate calamine;
extern crate csv;

mod sd3;
mod mifc;
mod units;
mod traits;

use failure::{Error, ResultExt};
use structopt::StructOpt;
use calamine::{Reader, RangeDeserializerBuilder, open_workbook_auto};
use std::path::PathBuf;
use std::fs::OpenOptions;
use std::ffi::{OsStr};
use sd3::SD3;

#[derive(StructOpt, Debug)]
#[structopt(name = "sd3norm")]
struct Opt {
    /// Input sd3-formatted excel file
    #[structopt(parse(from_os_str))]
    input: PathBuf,
    /// Output file name, adds "-normalized" to INPUT if blank
    #[structopt(parse(from_os_str))]
    output: Option<PathBuf>,
}

fn main() {
    let opts = Opt::from_args();

    if let Err(e) = run(opts) {
        print_err(&e);
        match ::std::env::var("RUST_BACKTRACE").as_ref().map(|s| s.as_str()) {
            Ok("1") => eprintln!("Backtrace:\n{}", e.backtrace()),
            _ => (),
        }
        ::std::process::exit(1);
    }
}

fn run(opts: Opt) -> Result<(), Error> {
    /* Get output base path, and read input excel workbook */
    let output_base = if let Some(path) = opts.output {
        path
    } else {
        let mut output = opts.input.clone();
        append_file_name(&mut output, "-normalized");
        output.set_extension("csv");
        output
    };
    let mut workbook = open_workbook_auto(opts.input)
        .context("opening input xlsx workbook")?;
    /* Iterate over the sheets in a workbook */
    // TODO: add sheet flag to CLI, and only process that sheet.
    let sheets = workbook.sheet_names().to_vec();
    let sheet_sum = sheets.len();

    for (i, s) in sheets.iter().enumerate() {
        let sheet = workbook.worksheet_range(s).unwrap()?;
        /* Generate a writer to output the normalized values from this sheet */
        let output = if sheet_sum == 1 {
                output_base.clone() // shouldn't need this clone, how to rewrite...
            } else {
                let mut out = output_base.clone();
                append_file_name(&mut out, format!("-{}",s));
                out
            };
        println!("Sheet #{}: {}\nOutput file: {:?}", i, s, &output);
        let mut wtr = csv::Writer::from_writer(
            OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&output)?
        );

        /* Deserialize the data into SD3 struct, then normalize each possible row*/
        let mut rows = RangeDeserializerBuilder::new()
            .has_headers(true)
            .from_range(&sheet)
            .context(format!("parsing sheet <{}>", s))?;

        for (i, result) in rows.enumerate() {
            let record: SD3 = match result {
                Ok(r) => r,
                Err(e) => {
                    println!("couldn't deserializing row {} in {}:\n{}", i+2, s, e); 
                    continue;
                },
            };

            let normalized = match record.into_normalized() {
                Ok(n) => n,
                Err(e) => {
                    println!("couldn't normalize row {} in {}:\n{}", i+2, s, e);
                    continue;
                },
            };

            wtr.serialize(normalized)?;
        }
    }

    Ok(())
}

fn print_err(e: &Error) {
    eprintln!("Error: {}", e);
    for e in e.causes().skip(1) {
        eprintln!("caused by: {}", e);
    }
}

fn append_file_name<S: AsRef<OsStr>>(path: &mut PathBuf, append: S) {
    if path.file_name().is_some() {
        let appended = { 
            let mut new = path.file_stem().unwrap().to_os_string();
            new.push(append);
            if let Some(e) = path.extension() {
                new.push("."); new.push(e);
            }
            new
        };
        path.set_file_name(appended);
    } else {
        path.set_file_name(append);
    }
}

