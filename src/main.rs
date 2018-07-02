#[macro_use] extern crate structopt;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate failure;
extern crate serde;
extern crate calamine;

mod sd3;
mod units;
mod traits;

use failure::{Error, ResultExt};
use structopt::StructOpt;
use calamine::{Reader, RangeDeserializerBuilder, open_workbook_auto};
use std::path::PathBuf;
use sd3::SD3;

#[derive(StructOpt, Debug)]
#[structopt(name = "sd3norm")]
struct Opt {
    /// Input sd3-formatted excel file
    #[structopt(parse(from_os_str))]
    input: PathBuf,
    /// Output file name
    #[structopt(parse(from_os_str))]
    output: Option<PathBuf>,
}

fn main() {
    let opts = Opt::from_args();

    println!("{:?}", opts);

    if let Err(e) = run(opts) {
        eprintln!("Error: {}", e);
        for e in e.causes().skip(1) {
            eprintln!("caused by: {}", e);
        }

        match ::std::env::var("RUST_BACKTRACE").as_ref().map(|s| s.as_str()) {
            Ok("1") => eprintln!("Backtrace: {}", e.backtrace()),
            _ => (),
        }

        ::std::process::exit(1);
    }
}

fn run(opts: Opt) -> Result<(), Error> {
    let mut workbook = open_workbook_auto(opts.input)
        .context("opening input xlsx workbook")?;

    println!("Sheets: {:#?}", workbook.sheet_names());
    let sheets = workbook.sheet_names().to_vec();

    for (i, s) in sheets.iter().enumerate() {
        println!("Sheet #{}: {}", i, s);
        let sheet = workbook.worksheet_range(s).unwrap()?;

        let mut rows = RangeDeserializerBuilder::new()
            .has_headers(true)
            .from_range(&sheet)?;

        for (i, result) in rows.take(5).enumerate() {
            let record: SD3 = result
                .context(format!("deserializing row {}", i+2))?;
            //println!("{:?}", &record);

            let normalized = match record.into_normalized() {
                Ok(n) => n,
                Err(e) => {
                    println!("Row {} couldn't be normalized due to:\n{}", i+2, e);
                    continue;
                },
            };

            println!("normalized:\n{:#?}", &normalized);
        }

    }
    Ok(())
}

