use std::path::PathBuf;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "example", about = "An example of StructOpt usage.")]
struct Opt {
    /// Input image, can be any common image file `image` can parse.
    input: PathBuf,
    /// Destination KTX2 file
    output: PathBuf,
}

pub fn main() {
    let opt = Opt::from_args();
    dbg!(opt);
}
