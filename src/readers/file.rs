
use std::path;
use std::io::{self, BufRead};
use std::fs;

use super::errors::ReadError;
use super::InputSource;

pub struct FileOpener;

impl InputSource for FileOpener {
    fn open(&mut self, path : &path::Path) -> Result<Box<Iterator<Item=Result<char,ReadError>>>, ReadError> {
        let file = match fs::File::open(path) {
            Ok(f) => f,
            Err(e) => return Err(ReadError::IOError(e)),
        };
        Ok(Box::new(FileReader::new(io::BufReader::new(file))))
    }
}

pub struct FileReader {
    buf : io::BufReader<fs::File>,
    chars : Option<Vec<char>>,
    pos : usize,
}

impl FileReader {
    pub fn new(buf : io::BufReader<fs::File>) -> FileReader {
        FileReader {
            buf : buf,
            chars : None,
            pos : 0,
        }
    }
}

impl Iterator for FileReader {
    type Item = Result<char,ReadError>;
    
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // try to read a char from current line
            let ret = match self.chars {
                Some(ref chars) => match chars.get(self.pos) {
                    Some(&ch) => Some(ch),
                    None => None,
                },
                None => None,
            };
            
            match ret {
                Some(ch) => {
                    self.pos += 1;
                    return Some(Ok(ch))
                }
                
                None => {
                    // read new line
                    let mut str = String::new();
                    match self.buf.read_line(&mut str) {
                        Err(e) => return Some(Err(ReadError::IOError(e))),
                        Ok(0) => return None,
                        Ok(_) => {},
                    }
                    self.chars = Some(str.chars().collect());
                    self.pos = 0;
                }
            }
        }
    }
}
