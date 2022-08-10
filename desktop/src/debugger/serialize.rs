use super::ServerMessageKind;
use num_traits::ToPrimitive;
use std::{ffi::OsStr, io::Write};

pub trait DebuggerSerialize {
    fn debug_serialize(&self, output: &mut impl Write) -> std::io::Result<()>;
}

impl<'a> DebuggerSerialize for &'a [u8] {
    fn debug_serialize(&self, output: &mut impl Write) -> std::io::Result<()> {
        output.write_all(self)?;
        output.write_all(b"\x00")
    }
}

impl<'a> DebuggerSerialize for &'a str {
    fn debug_serialize(&self, output: &mut impl Write) -> std::io::Result<()> {
        self.as_bytes().debug_serialize(output)
    }
}

impl<'a> DebuggerSerialize for &'a OsStr {
    fn debug_serialize(&self, output: &mut impl Write) -> std::io::Result<()> {
        (&*self.to_string_lossy()).debug_serialize(output)
    }
}

impl DebuggerSerialize for usize {
    fn debug_serialize(&self, output: &mut impl Write) -> std::io::Result<()> {
        output.write_all(&self.to_le_bytes())
    }
}

impl DebuggerSerialize for u32 {
    fn debug_serialize(&self, output: &mut impl Write) -> std::io::Result<()> {
        output.write_all(&self.to_le_bytes())
    }
}

impl DebuggerSerialize for u16 {
    fn debug_serialize(&self, output: &mut impl Write) -> std::io::Result<()> {
        output.write_all(&self.to_le_bytes())
    }
}

impl DebuggerSerialize for u8 {
    fn debug_serialize(&self, output: &mut impl Write) -> std::io::Result<()> {
        output.write_all(&self.to_le_bytes())
    }
}

pub struct DebugBuilder {
    kind: ServerMessageKind,
    data: Vec<u8>,
}

impl DebugBuilder {
    pub fn new(kind: ServerMessageKind) -> Self {
        Self {
            kind,
            data: Vec::new(),
        }
    }

    pub fn add(&mut self, f: impl DebuggerSerialize) {
        f.debug_serialize(&mut self.data).expect("Write failed");
    }

    pub fn send(self, dst: &mut impl Write) -> std::io::Result<()> {
        dst.write_all(&(self.data.len() as u32).to_le_bytes())?;
        dst.write_all(&self.kind.to_u32().unwrap().to_le_bytes())?;
        dst.write_all(&self.data)
    }
}
