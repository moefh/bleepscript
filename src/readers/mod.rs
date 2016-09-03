
mod string;
mod file;
mod errors;

use std::path;

pub use self::errors::ReadError;
pub use self::file::FileOpener;
pub use self::string::{StringOpener, StringCharReader};

pub trait CharReader {
    
    /// Returns the line number of the next character.
    fn line_num(&self) -> u32;

    /// Returns the column number of the next character.
    fn col_num(&self) -> u32;

    /// Returns the next character, a `ReadError` on error, or `None` if the end of the stream was reached. 
    fn getc(&mut self) -> Option<Result<char, ReadError>>;
    
    /// Pushes the given character back to the stream, so the next call to `getc()` will return it.
    fn ungetc(&mut self, ch : char);
}

pub trait CharReaderOpener {
    /// Opens the given source and returns a `CharReader` to read characters from it. 
    fn open(&mut self, source : &path::Path) -> Result<Box<CharReader>, ReadError>;
}

