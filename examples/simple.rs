use cling::prelude::*;

#[derive(Parser)]
struct Opts {
    /// name of project
    name: String,
}

fn main() {
    Opts::parse();
    println!("Hello, world!");
}
