use clap::Parser;
use rad::Options;

fn main() {
    let opts = Options::parse();
    println!("{opts:#?}");
}
