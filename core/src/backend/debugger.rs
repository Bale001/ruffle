pub trait DebuggerBackend {
    fn tick(&mut self) -> Option<bool>;
    fn connect(&mut self, password: &str, port: u16) -> bool;
    fn on_position(&mut self, pos: u32) -> bool;
}

pub struct NullDebuggerBackend;

impl DebuggerBackend for NullDebuggerBackend {
    fn tick(&mut self) -> Option<bool> {
        None
    }

    fn connect(&mut self, _password: &str, _port: u16) -> bool {
        false
    }

    fn on_position(&mut self, _pos: u32) -> bool {
        false
    }
}
