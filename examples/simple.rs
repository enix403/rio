use std::io::Cursor;

fn main() {
    // let mut rio = rio::Rio::new(Cursor::new(String::from("4 2 hello")));
    let mut rio = rio::Rio::new(std::io::stdin());

    let a: i8 = rio.read_or_default();
    let b: u32 = rio.read_or_default();

    println!("a = {}", a);
    println!("b = {}", b);
}

// fn main() {
//     let mut rio = rio::Rio::new(Cursor::new(String::from("4 2 hello")));

//     let a: u32 = rio.read_or_default();
//     let b: u32 = rio.read_or_default();

//     let name: String = rio.read_or_default();

//     println!("name = {}", name);
//     println!("n = {}, d = {}", a, b);
// }
