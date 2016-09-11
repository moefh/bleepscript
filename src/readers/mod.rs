
mod string;
mod file;
mod errors;

use std::path;

pub use self::errors::ReadError;
pub use self::file::FileOpener;
pub use self::string::StringInputOpener;

pub trait InputSource {
    /// Opens the given source and returns an `Iterator` that yields the characters of the source.
    fn open(&mut self, source : &path::Path) -> Result<Box<Iterator<Item=Result<char,ReadError>>>, ReadError>;
}
