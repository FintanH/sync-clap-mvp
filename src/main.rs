use clap::Parser;
use sync::Options;

fn main() {
    let opts = Options::parse();
    println!("{opts:#?}");
}
