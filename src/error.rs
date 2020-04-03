#[derive(Debug)]
pub enum Error {
    /// Error opening an IDB database
    IdbOpen,
    /// Invalid IDB database version
    IdbVersion,
}

pub type Result<T> = std::result::Result<T, Error>;
