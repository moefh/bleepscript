
use std::fmt;
use std::rc::Rc;

#[derive(Clone)]
pub struct SrcLoc {
    pub filename : Rc<String>,
    pub line_num : u32,
    pub col_num : u32,
}

impl SrcLoc {
    pub fn new(filename : &str, line_num : u32, col_num : u32) -> SrcLoc {
        SrcLoc {
            filename : Rc::new(filename.to_string()),
            line_num : line_num,
            col_num : col_num,
        }
    }
    
    pub fn new_at(&self, line_num : u32, col_num : u32) -> SrcLoc {
        SrcLoc {
            filename : self.filename.clone(),
            line_num : line_num,
            col_num : col_num,
        }
    }
}

impl fmt::Display for SrcLoc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}:{}", self.filename, self.line_num, self.col_num)
    }
}
