use std::sync::Arc;

use crate::tag_utils::SwfMovie;

pub trait DebuggerBackend {
    fn tick(&mut self) -> Option<bool>;
    fn connect(&mut self, password: &str, port: u16) -> bool;
    fn add_movie(&mut self, movie: Arc<SwfMovie>);
}

pub struct NullDebuggerBackend;

impl DebuggerBackend for NullDebuggerBackend {
    fn tick(&mut self) -> Option<bool> {
        None
    }

    fn connect(&mut self, _password: &str, _port: u16) -> bool {
        false
    }

    fn add_movie(&mut self, _movie: Arc<SwfMovie>) {}
}
