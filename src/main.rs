#[macro_use] extern crate structopt;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate failure;
#[macro_use] extern crate log;
extern crate flexi_logger;
extern crate serde;
extern crate calamine;
extern crate csv;

mod sd3;
mod mifc;
mod si;
#[cfg(test)] mod utils;

use failure::{Error, ResultExt};
use structopt::StructOpt;
use calamine::{Reader, RangeDeserializerBuilder, open_workbook_auto};
use flexi_logger::{Logger, default_format};
use std::path::{Path, PathBuf};
use std::fmt;
use std::fs::{OpenOptions, self};
use std::ffi::{OsStr};
use sd3::SD3;

#[derive(StructOpt, Debug)]
/// Read an SD3 (MIFC + normalization info) excel workbook and create one normalized MIFC CSV for each sheet
struct Opt {
    /// Input sd3-formatted excel file
    #[structopt(parse(from_os_str))]
    input: PathBuf,
    /// Append to INPUT file name when naming output file, defaults to "normalized"
    append: Option<String>,
    /// Print debug info based on the number of "v"s passed
    #[structopt(short = "v", parse(from_occurrences))]
    verbose: usize,
    /// Optional directory for output
    #[structopt(short = "d", long = "out-dir", parse(from_os_str))]
    out_dir: Option<PathBuf>, 
}

fn main() {
    let opts = Opt::from_args();
    let log_level = match opts.verbose {
        0 => "error",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };

    Logger::with_str(log_level)
        .format(default_format)
        .start()
        .unwrap_or_else(|e| panic!("Logger initialization failed with {}",e) );

    if let Err(e) = run(opts) {
        print_err(&e);
        match ::std::env::var("RUST_BACKTRACE").as_ref().map(|s| s.as_str()) {
            Ok("1") => error!("Backtrace:\n{}", e.backtrace()),
            _ => (),
        }
        ::std::process::exit(1);
    }
}

fn run(opts: Opt) -> Result<(), Error> {
    /* Convert collection of input files or directories into workbooks paths */
    let workbook = opts.input;
    debug!("Workbooks: {:#?}", &workbook);

    /* Get output base path by appending the value of optional directory flag */
    let output_base = if let Some(mut dir) = opts.out_dir {
        if !dir.exists() { 
            fs::create_dir_all(&dir)?;
        } else if !dir.is_dir() { 
            bail!("Path <{:?}> passed to \"--out-dir\" is not a directory", &dir);
        }

        debug!("Output directory: {:?}", dir);

        let input_filename = workbook.file_name()
            .expect("the input workbook was not a file");

        dir.push(input_filename);
        dir.set_extension("csv");

        dir
    } else {
        workbook.with_extension("csv")
    };

    /* Get the value to append to the end of the output, or use the default */
    let append_str = opts.append.as_ref().map_or("normalized", String::as_ref);
    debug!("output base: {:?}\nappend: {}", output_base, &append_str);

    normalize_workbook(&workbook, &output_base, &append_str)?;

    Ok(())
}

fn normalize_workbook<P>(wb_path: P, output_base: P, append: &str) -> Result<(), Error>
where P: AsRef<Path> + fmt::Debug
{
    let mut workbook = open_workbook_auto(&wb_path)
        .context(format!("opening excel workbook <{:?}>", &wb_path))?;
    /* Iterate over the sheets in a workbook */
    // TODO: add sheet flag to CLI, and only process that sheet.
    let sheets = workbook.sheet_names().to_vec();
    let sheet_sum = sheets.len();

    for (i, s) in sheets.iter().enumerate() {
        let sheet = workbook.worksheet_range(s).unwrap()?;

        /* Generate a writer to output the normalized values from this sheet 
         * If there is only one sheet, don't append the sheet name to the output file name
        **/
        let output = {
            let mut out = output_base.as_ref().to_path_buf();
            let add_sheet = sheet_sum > 1;
            let appended_info = format!("{s_h}{s}-{a}", 
                s_h = if add_sheet {"-"} else {""},
                s = if add_sheet {s} else {""},
                a =  append
            );
            append_file_name(&mut out, &appended_info);
            out
        };
        
        info!("{:?} - {} (#{}):\nOutput file: {:?}", &wb_path, s, i, &output);

        let mut wtr = csv::Writer::from_writer(
            OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&output)?
        );

        /* Deserialize the data into SD3 struct, then normalize each possible row, and serialize into output*/
        let mut rows = match RangeDeserializerBuilder::new()
            .has_headers(true)
            .from_range(&sheet)
        {
            Ok(r) => r,
            Err(e) => {
                warn!("issue parsing sheet <{}>\n{}", s, e);
                continue;
            } 
        };

        for (i, result) in rows.enumerate() {
            let record: SD3 = match result {
                Ok(r) => r,
                Err(e) => {
                    info!("couldn't deserializing row {} in {}:\n{}", i+2, s, e); 
                    continue;
                },
            };

            let normalized = match record.into_normalized() {
                Ok(n) => n,
                Err(e) => {
                    info!("did not normalize row {} in {}:\n{}", i+2, s, e);
                    continue;
                },
            };

            wtr.serialize(normalized)?;
        }
    }
    Ok(())
}

fn print_err(e: &Error) {
    error!(": {}", e);
    for e in e.causes().skip(1) {
        error!("caused by: {}", e);
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

