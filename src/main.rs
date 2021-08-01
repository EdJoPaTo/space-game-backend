use std::time::Instant;

mod statics;

fn main() {
    println!("Hello, world!");

    println!("load static data...");
    let bla = Instant::now();
    let statics = statics::Statics::import("../typings/static").unwrap();
    let took = bla.elapsed();
    println!("took {:?}", took);
    println!("statics {:?}", statics);
}
