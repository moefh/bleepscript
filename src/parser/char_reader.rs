
use std::io;
use super::errors::ReadError;

pub struct CharReader<Buf : io::BufRead> {
    buf : Buf,
    line_num : u32,
    col_num : u32,
    col_num_before_newline : u32,
    chars : Option<Vec<char>>,
    pos : usize,
    saved : Vec<char>,
}

impl<Buf : io::BufRead> CharReader<Buf> {
    pub fn new(buf : Buf) -> CharReader<Buf> {
        CharReader {
            buf : buf,
            line_num : 1,
            col_num : 1,
            col_num_before_newline : 0,
            chars : None,
            pos : 0,
            saved : vec![],
        }
    }
    
    pub fn ungetc(&mut self, ch : char) {
        self.retreat_loc(ch);
        self.saved.push(ch);
    }
    
    pub fn line_num(&self) -> u32 {
        self.line_num
    }

    pub fn col_num(&self) -> u32 {
        self.col_num
    }
    
    fn advance_loc(&mut self, ch : char) {
        if ch == '\n' {
            self.col_num_before_newline = self.col_num;
            self.line_num += 1;
            self.col_num = 1;
        } else {
            self.col_num += 1;
        }
    }

    fn retreat_loc(&mut self, ch : char) {
        if ch == '\n' {
            self.line_num -= 1;
            self.col_num = self.col_num_before_newline;
        } else {
            self.col_num -= 1;
        }
    }
    
    pub fn next(&mut self) -> Option<Result<char, ReadError>> {
        
        if let Some(ch) = self.saved.pop() {
            self.advance_loc(ch);
            return Some(Ok(ch));
        }
        
        loop {
            // try to read a char
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
                    self.advance_loc(ch);
                    return Some(Ok(ch))
                }
                
                None => {
                    // read new line
                    let mut str = String::new();
                    match self.buf.read_line(&mut str) {
                        Err(e) => return Some(Err(ReadError::IOError(e))),
                        Ok(0) => { self.col_num += 1; return None; }
                        Ok(_) => {},
                    }
                    self.chars = Some(str.chars().collect());
                    self.pos = 0;
                    //self.col_num = 1;
                }
            }
        }
    }
}
