pub trait DebuggerBackend {
    fn tick(&mut self) -> Option<()>;
    fn connect(&mut self, password: &str, port: u16) -> bool;
}

pub struct NullDebuggerBackend;

impl DebuggerBackend for NullDebuggerBackend {
    fn tick(&mut self) -> Option<()> {
        None
    }

    fn connect(&mut self, _password: &str, _port: u16) -> bool {
        false
    }
}
