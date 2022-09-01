use super::ServerMessageKind;
use ruffle_core::{Avm2Callstack, string::WString, Avm2CallNode};
use ruffle_core::string::WStr;
use num_traits::ToPrimitive;
use std::ops::Deref;
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

impl<'a> DebuggerSerialize for &'a WStr {
    fn debug_serialize(&self, output: &mut impl Write) -> std::io::Result<()> {
        let s = self.to_utf8_lossy();
        s.as_ref().debug_serialize(output)
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

impl DebuggerSerialize for bool {
    fn debug_serialize(&self, output: &mut impl Write) -> std::io::Result<()> {
        (*self as u8).debug_serialize(output)
    }
}

impl<'a, 'gc> DebuggerSerialize for &'a Avm2Callstack<'gc> {
    fn debug_serialize(&self, output: &mut impl Write) -> std::io::Result<()> {
        let nodes = self.nodes();
        output.write_all(&(nodes.len() as u32).to_le_bytes())?;
        for node in self.nodes().iter().rev() {
            output.write_all(&u16::MAX.to_le_bytes())?;
            output.write_all(&0u16.to_le_bytes())?;
            output.write_all(&0usize.to_le_bytes())?;
            match node {
                Avm2CallNode::GlobalInit => "global$init".debug_serialize(output)?,
                Avm2CallNode::Method(exec) => {
                    let mut name = WString::new();
                    exec.write_full_name(&mut name);
                    name.deref().debug_serialize(output)?;
                }
            };
        }
        Ok(())
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
