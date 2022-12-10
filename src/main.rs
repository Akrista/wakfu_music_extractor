extern crate byteorder;
extern crate failure;

pub mod package;

use failure::{err_msg, Error, ResultExt};

use std::{env, fs::File, io::BufReader, process::exit};

use package::{Header, OggFile, OggInfo, OggName};

fn app() -> Result<(), Error> {
    let package_path = env::args()
        .skip(1)
        .next()
        .ok_or_else(|| err_msg("You need to specify the .pk file to extract from!"))?;

    println!("open '{}'", package_path);
    let mut package = BufReader::new(
        File::open(&package_path).with_context(|_| format!("Could not open '{}'", package_path))?,
    );
    let header = Header::read_from(&mut package).context("Could not read package header")?;
    let offsets = header.offsets();

    println!("{} files to unpack", offsets.len());
    for (nr, &offset) in offsets.iter().enumerate() {
        let name = OggName::new(nr as u32);
        println!("unpacking {}", name);
        let ogg = OggFile::read_from(&mut package, OggInfo::new(name, offset))
            .with_context(|_| format!("Could not read file '{}'", name))?;
        ogg.write_to_file()?;
    }

    println!("Done");
    Ok(())
}

fn main() {
    if let Err(err) = app() {
        eprintln!("ERROR: {}", err);
        for err in err.iter_chain().skip(1) {
            eprintln!("Cause: {}", err)
        }
        exit(1)
    }
}
