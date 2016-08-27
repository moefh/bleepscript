
use super::super::SrcLoc;
use super::super::parser::{ParseResult, ParseError};

#[derive(Clone)]
struct InternalState {
    allow_break : bool,
}

impl InternalState {
    pub fn new() -> InternalState {
        InternalState {
            allow_break : false,
        }
    }
}

pub struct State {
    states : Vec<InternalState>
}

impl State {
    pub fn new() -> State {
        State {
            states : vec![InternalState::new(); 1]
        }
    }
    
    pub fn save_state(&mut self) {
        self.states.push(InternalState::new());
    }

    pub fn restore_state(&mut self, loc : &SrcLoc) -> ParseResult<()> {
        match self.states.pop() {
            Some(_) => Ok(()),
            None => Err(ParseError::new(loc.clone(), "invalid anaylsis state"))
        }
    }

    pub fn allow_break(&self, loc : &SrcLoc) -> ParseResult<bool> {
        match self.states.last() {
            Some(st) => Ok(st.allow_break),
            None => Err(ParseError::new(loc.clone(), "invalid anaylsis state"))
        }
    }
    
    pub fn set_allow_break(&mut self, allow_break : bool, loc : &SrcLoc) -> ParseResult<()> {
        match self.states.last_mut() {
            Some(st) => { st.allow_break = allow_break; Ok(()) }
            None => Err(ParseError::new(loc.clone(), "invalid anaylsis state"))
        }
    }

}
