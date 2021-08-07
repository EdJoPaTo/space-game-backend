mod effect;
pub mod round;
pub mod ship;

#[cfg(test)]
fn get_statics() -> typings::fixed::Statics {
    typings::fixed::Statics::import_yaml("../typings/static").unwrap()
}
