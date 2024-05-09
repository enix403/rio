#[derive(Debug)]
pub enum InputError {
    IO(std::io::Error),
    Extraction,
}
