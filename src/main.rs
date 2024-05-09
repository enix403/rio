#![allow(unused)]
#![allow(unused_variables)]
#![allow(unused_mut)]

mod utils;
mod error;
mod rio;
mod stream;

use std::io::Read;
use stream::RioStream;

fn main() {
    let mut rio = rio::Rio::new(std::io::stdin().bytes());

    let name: String = rio.read_or_default();

    let n: u32 = rio.read_or_default();
    let d: u32 = rio.read_or_default();

    let values: Vec<u32> = rio.read_n_or_default::<u32, _>(n);

    println!("name = {}", name);
    println!("n = {}, d = {}", n, d);
    println!("values = {:?}", values);
}
