use std::sync::Arc;

use crate::{tag_utils::SwfMovie, avm2::CallStack};

pub trait DebuggerBackend {
    fn tick(&mut self) -> Option<bool>;
    fn connect(&mut self, password: &str, port: u16) -> bool;
    fn add_movie(&mut self, movie: Arc<SwfMovie>);
    fn on_script_loaded(&mut self, call_stack: &CallStack<'_>);
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
    fn on_script_loaded(&mut self, _call_stack: &CallStack<'_>) {}
}
