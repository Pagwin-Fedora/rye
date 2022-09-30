extern crate reqwest;
extern crate lazy_static;
extern crate clap;
extern crate serde;
extern crate toml;
use lazy_static::lazy_static;
lazy_static!{
    static ref HTTP:reqwest::blocking::Client = reqwest::blocking::Client::new();
}
fn main() {
    
}
