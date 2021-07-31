use std::time::Instant;

mod types;

fn main() {
    println!("Hello, world!");

    println!("load static data...");
    let bla = Instant::now();
    let statics = types::parse_static().unwrap();
    let took = bla.elapsed();
    println!("took {:?}", took);
    println!("statics {:?}", statics);
}
