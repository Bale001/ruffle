mod message;
mod serialize;

use bytes::{Buf, Bytes, BytesMut};
use message::{ClientMessageKind, ServerMessageKind};
use num_traits::cast::FromPrimitive;
use ruffle_core::backend::debugger::DebuggerBackend;
use serialize::DebugBuilder;
use std::io::Read;
use std::net::TcpStream;
use url::Url;

macro_rules! send_debug {
    ($stream: expr, $kind: expr) => {
        if let Some(stream) = &mut $stream {
            let _ = DebugBuilder::new($kind).send(stream);
        }
    };
    ($stream: expr, $kind: expr, $($field:expr),+) => {
        if let Some(stream) = &mut $stream {
            let mut builder = DebugBuilder::new($kind);
            $(
                builder.add($field);
            )+
            let _ = builder.send(stream);
        }
    };
}

#[allow(dead_code)]
#[derive(Default)]
struct DebuggerProperties {
    astrace: u32,
    break_on_fault: bool,
    console_errors: bool,
    disable_script_stuck: bool,
    disable_script_stuck_dialog: bool,
    enumerate_override: bool,
    getter_timeout: u32,
    invoke_setters: bool,
    notify_on_failure: bool,
    setter_timeout: u32,
    script_timeout: u32,
    swf_load_messages: bool,
    verbose: bool,
}

pub struct RemoteDebuggerBackend {
    stream: Option<TcpStream>,
    url: Option<Url>,

    properties: DebuggerProperties,
    squelch: bool,

    packet_kind: Option<ClientMessageKind>,
    data: BytesMut,
}

fn bool_to_str(b: bool) -> &'static str {
    match b {
        true => "true",
        false => "false",
    }
}

impl RemoteDebuggerBackend {
    pub fn new(file_url: Option<Url>) -> Self {
        Self {
            stream: None,
            url: file_url,
            properties: DebuggerProperties::default(),
            squelch: false,
            packet_kind: None,
            data: BytesMut::new(),
        }
    }

    fn read_header(&mut self) -> Option<(u32, ClientMessageKind)> {
        let stream = self.stream.as_mut()?;
        let mut buf = [0; 8];

        stream.read_exact(&mut buf).ok()?;
        let length = u32::from_le_bytes(buf[..4].try_into().unwrap());
        let message_kind =
            ClientMessageKind::from_u32(u32::from_le_bytes(buf[4..].try_into().unwrap()))?;

        Some((length, message_kind))
    }

    fn execute(&mut self, kind: ClientMessageKind) -> Option<()> {
        match kind {
            ClientMessageKind::SetDebugOption => self.set_debug_option()?,
            ClientMessageKind::SetSquelch => self.set_squelch()?,
            _ => println!("Unhandled message kind: {:?}", kind),
        }
        Some(())
    }

    fn read_string(&mut self) -> Option<Bytes> {
        let null_at = self.data.iter().position(|b| *b == b'\0')?;
        let string = self.data.split_to(null_at);
        self.data.advance(1);
        Some(string.freeze())
    }

    fn read_u32(&mut self) -> Option<u32> {
        if self.data.len() < 4 {
            return None;
        }
        let num = self.data.split_to(4).freeze();
        Some(u32::from_le_bytes((*num).try_into().unwrap()))
    }

    fn read_switch(&mut self) -> Option<bool> {
        let prop = self.read_string()?;
        match &*prop {
            b"on" => Some(true),
            b"off" => Some(false),
            _ => None,
        }
    }

    fn set_squelch(&mut self) -> Option<()> {
        self.squelch = self.read_u32()? != 0;
        send_debug!(self.stream, ServerMessageKind::Squelch, self.squelch as u32);
        Some(())
    }

    fn set_debug_option(&mut self) -> Option<()> {
        let prop = self.read_string()?;
        match &*prop {
            b"disable_script_stuck_dialog" => {
                self.properties.disable_script_stuck_dialog = self.read_switch()?;
                send_debug!(
                    self.stream,
                    ServerMessageKind::DebuggerOption,
                    "disable_script_stuck_dialog",
                    bool_to_str(self.properties.disable_script_stuck_dialog)
                )
            }
            b"disable_script_stuck" => {
                self.properties.disable_script_stuck = self.read_switch()?;
                send_debug!(
                    self.stream,
                    ServerMessageKind::DebuggerOption,
                    "disable_script_stuck",
                    bool_to_str(self.properties.disable_script_stuck)
                )
            }
            b"break_on_fault" => {
                self.properties.break_on_fault = self.read_switch()?;
                send_debug!(
                    self.stream,
                    ServerMessageKind::DebuggerOption,
                    "break_on_fault",
                    bool_to_str(self.properties.break_on_fault)
                )
            }
            b"enumerate_override" => {
                self.properties.enumerate_override = self.read_switch()?;
                send_debug!(
                    self.stream,
                    ServerMessageKind::DebuggerOption,
                    "enumerate_override",
                    bool_to_str(self.properties.enumerate_override)
                )
            }
            b"notify_on_failure" => {
                self.properties.notify_on_failure = self.read_switch()?;
                send_debug!(
                    self.stream,
                    ServerMessageKind::DebuggerOption,
                    "notify_on_failure",
                    bool_to_str(self.properties.notify_on_failure)
                )
            }
            b"invoke_setters" => {
                self.properties.invoke_setters = self.read_switch()?;
                send_debug!(
                    self.stream,
                    ServerMessageKind::DebuggerOption,
                    "invoke_setters",
                    bool_to_str(self.properties.invoke_setters)
                )
            }
            b"swf_load_messages" => {
                self.properties.swf_load_messages = self.read_switch()?;
                send_debug!(
                    self.stream,
                    ServerMessageKind::DebuggerOption,
                    "swf_load_messages",
                    bool_to_str(self.properties.swf_load_messages)
                )
            }
            b"getter_timeout" => {
                let prop = self.read_string()?;
                let value = std::str::from_utf8(&prop).ok()?;
                self.properties.getter_timeout = value.parse().ok()?;
                send_debug!(
                    self.stream,
                    ServerMessageKind::DebuggerOption,
                    "getter_timeout",
                    value
                )
            }
            b"setter_timeout" => {
                let prop = self.read_string()?;
                let value = std::str::from_utf8(&prop).ok()?;
                self.properties.setter_timeout = value.parse().ok()?;
                send_debug!(
                    self.stream,
                    ServerMessageKind::DebuggerOption,
                    "setter_timeout",
                    value
                )
            }
            _ => (),
        }
        Some(())
    }
}

impl DebuggerBackend for RemoteDebuggerBackend {
    fn tick(&mut self) -> Option<()> {
        if let Some(stream) = self.stream.as_mut() {
            if let Some(kind) = self.packet_kind {
                stream.read_exact(&mut self.data).ok()?;
                self.packet_kind = None;
                self.execute(kind);
                self.data.clear();
            } else {
                let (length, kind) = self.read_header()?;
                self.data.resize(length as usize, 0);
                self.packet_kind = Some(kind);
            }
        }
        None
    }

    fn connect(&mut self, password: &str, port: u16) -> bool {
        if let Ok(stream) = TcpStream::connect(("127.0.0.1", port)) {
            stream
                .set_nonblocking(true)
                .expect("failed to set debug stream as nonblocking");
            self.stream = Some(stream);
            let movie_url = self.url.take().unwrap();
            send_debug!(self.stream, ServerMessageKind::SetVersion, 0x0fu32);
            send_debug!(
                self.stream,
                ServerMessageKind::MovieAttribute,
                "movie",
                movie_url.as_str()
            );
            send_debug!(
                self.stream,
                ServerMessageKind::MovieAttribute,
                "password",
                password
            );
            true
        } else {
            false
        }
    }
}
