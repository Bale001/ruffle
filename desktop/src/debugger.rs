mod message;
mod serialize;

use bytes::{Buf, Bytes, BytesMut};
use message::{ClientMessageKind, ServerMessageKind};
use num_traits::cast::FromPrimitive;
use ruffle_core::backend::debugger::DebuggerBackend;
use serialize::DebugBuilder;
use std::net::TcpStream;
use std::path::PathBuf;
use std::{fs::File, io::Read};
use swd_rs::{Swd, SwdReader};

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
    wide_line_player: bool,
    wide_line_debugger: bool,
}

fn display_message(message: &str) {
    let dialog = rfd::MessageDialog::new()
        .set_level(rfd::MessageLevel::Info)
        .set_title("Ruffle")
        .set_description(message)
        .set_buttons(rfd::MessageButtons::Ok);
    dialog.show();
}

fn read_swd(path: PathBuf) -> Option<Swd> {
    let file = File::open(path).ok()?;
    SwdReader::new(file).read().ok()
}

pub struct RemoteDebuggerBackend {
    stream: Option<TcpStream>,
    path: PathBuf,
    swd: Option<Swd>,

    properties: DebuggerProperties,
    squelch: bool,

    packet_kind: Option<ClientMessageKind>,
    data: BytesMut,
}

impl RemoteDebuggerBackend {
    pub fn new(file_url: PathBuf) -> Self {
        let swd_url = file_url.with_extension("swd");
        //std::fs::copy(&file_url, "/Users/haydencurtis/Desktop/rust/ruffle/test_avm2_b.swd").unwrap();
        Self {
            stream: None,
            path: file_url,
            swd: read_swd(swd_url),
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

    fn execute(&mut self, kind: ClientMessageKind) -> Option<bool> {
        match kind {
            ClientMessageKind::SetDebugOption => self.set_debug_option()?,
            ClientMessageKind::GetDebugOption => self.get_debug_option()?,
            ClientMessageKind::SetSquelch => self.set_squelch()?,
            ClientMessageKind::Continue => return Some(true),
            _ => display_message(&format!("Unknown message {:?}", kind)),
        }
        Some(false)
    }

    fn read_string(&mut self) -> Option<Bytes> {
        let null_at = self.data.iter().position(|b| *b == b'\0')?;
        let string = self.data.split_to(null_at);
        // consume null byte
        self.data.advance(1);
        Some(string.freeze())
    }

    fn read_u32(&mut self) -> Option<u32> {
        if self.data.len() < 4 {
            return None;
        }
        Some(self.data.get_u32_le())
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

    fn get_debug_option(&mut self) -> Option<()> {
        let prop = self.read_string()?;
        let val = match &*prop {
            b"disable_script_stuck_dialog" => {
                self.properties.disable_script_stuck_dialog.to_string()
            }
            b"disable_script_stuck" => self.properties.disable_script_stuck.to_string(),
            b"break_on_fault" => self.properties.break_on_fault.to_string(),
            b"enumerate_override" => self.properties.enumerate_override.to_string(),
            b"notify_on_failure" => self.properties.notify_on_failure.to_string(),
            b"invoke_setters" => self.properties.invoke_setters.to_string(),
            b"wide_line_player" => self.properties.wide_line_player.to_string(),
            b"wide_line_debugger" => self.properties.wide_line_debugger.to_string(),
            b"swf_load_messages" => self.properties.swf_load_messages.to_string(),
            b"getter_timeout" => self.properties.getter_timeout.to_string(),
            b"setter_timeout" => self.properties.setter_timeout.to_string(),
            _ => return None,
        };
        send_debug!(
            self.stream,
            ServerMessageKind::DebuggerOption,
            &*prop,
            &*val
        );
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
                    &*self.properties.disable_script_stuck_dialog.to_string()
                )
            }
            b"disable_script_stuck" => {
                self.properties.disable_script_stuck = self.read_switch()?;
                send_debug!(
                    self.stream,
                    ServerMessageKind::DebuggerOption,
                    "disable_script_stuck",
                    &*self.properties.disable_script_stuck.to_string()
                )
            }
            b"break_on_fault" => {
                self.properties.break_on_fault = self.read_switch()?;
                send_debug!(
                    self.stream,
                    ServerMessageKind::DebuggerOption,
                    "break_on_fault",
                    &*self.properties.break_on_fault.to_string()
                )
            }
            b"enumerate_override" => {
                self.properties.enumerate_override = self.read_switch()?;
                send_debug!(
                    self.stream,
                    ServerMessageKind::DebuggerOption,
                    "enumerate_override",
                    &*self.properties.enumerate_override.to_string()
                )
            }
            b"notify_on_failure" => {
                self.properties.notify_on_failure = self.read_switch()?;
                send_debug!(
                    self.stream,
                    ServerMessageKind::DebuggerOption,
                    "notify_on_failure",
                    &*self.properties.notify_on_failure.to_string()
                )
            }
            b"invoke_setters" => {
                self.properties.invoke_setters = self.read_switch()?;
                send_debug!(
                    self.stream,
                    ServerMessageKind::DebuggerOption,
                    "invoke_setters",
                    &*self.properties.invoke_setters.to_string()
                )
            }
            b"wide_line_player" => {
                self.properties.wide_line_player = self.read_switch()?;
                send_debug!(
                    self.stream,
                    ServerMessageKind::DebuggerOption,
                    "wide_line_player",
                    &*self.properties.wide_line_player.to_string()
                )
            }
            b"wide_line_debugger" => {
                self.properties.wide_line_debugger = self.read_switch()?;
                send_debug!(
                    self.stream,
                    ServerMessageKind::DebuggerOption,
                    "wide_line_debugger",
                    &*self.properties.wide_line_debugger.to_string()
                )
            }
            b"swf_load_messages" => {
                self.properties.swf_load_messages = self.read_switch()?;
                send_debug!(
                    self.stream,
                    ServerMessageKind::DebuggerOption,
                    "swf_load_messages",
                    &*self.properties.swf_load_messages.to_string()
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
    fn tick(&mut self) -> Option<bool> {
        let mut should_continue = false;
        if let Some(stream) = self.stream.as_mut() {
            if let Some(kind) = self.packet_kind {
                stream.read_exact(&mut self.data).ok()?;
                self.packet_kind = None;
                should_continue = self.execute(kind).unwrap_or(false);
                self.data.clear();
            } else {
                let (length, kind) = self.read_header()?;
                self.data.resize(length as usize, 0);
                self.packet_kind = Some(kind);
            }
        }
        Some(should_continue)
    }

    fn connect(&mut self, password: &str, port: u16) -> bool {
        if let Ok(stream) = TcpStream::connect(("127.0.0.1", port)) {
            stream
                .set_nonblocking(true)
                .expect("failed to set debug stream as nonblocking");
            self.stream = Some(stream);

            send_debug!(self.stream, ServerMessageKind::SetVersion, 0x0fu32);
            send_debug!(
                self.stream,
                ServerMessageKind::MovieAttribute,
                "movie",
                self.path.as_os_str()
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

    fn on_position(&mut self, pos: u32) -> bool {
        if let Some(swd) = self.swd.as_ref() {
            swd.resolve_breakpoint(pos).is_some()
        } else {
            false
        }
    }
}
