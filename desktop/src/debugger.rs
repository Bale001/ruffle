mod message;
mod serialize;

use bytes::{Buf, Bytes, BytesMut};
use message::{ClientMessageKind, ServerMessageKind};
use num_traits::cast::FromPrimitive;
use ruffle_core::backend::debugger::DebuggerBackend;
use ruffle_core::tag_utils::SwfMovie;
use serialize::{DebugBuilder, DebuggerSerialize};
use std::cell::RefCell;
use std::io::Read;
use std::net::TcpStream;
use std::path::PathBuf;
use std::sync::Arc;

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

struct DebugSender<'a> {
    builder: DebugBuilder,
    stream: &'a RefCell<Option<TcpStream>>,
}

impl<'a> DebugSender<'a> {
    fn arg(mut self, f: impl DebuggerSerialize) -> Self {
        if self.stream.borrow().is_some() {
            self.builder.add(f)
        }
        self
    }

    fn add(&mut self, f: impl DebuggerSerialize) {
        if self.stream.borrow().is_some() {
            self.builder.add(f)
        }
    }

    fn send(self) {
        if let Some(stream) = self.stream.borrow_mut().as_mut() {
            if let Err(e) = self.builder.send(stream) {
                log::warn!("Unable to send debug data: {}", e);
            }
        }
    }
}

fn display_message(message: &str) {
    let dialog = rfd::MessageDialog::new()
        .set_level(rfd::MessageLevel::Info)
        .set_title("Ruffle")
        .set_description(message)
        .set_buttons(rfd::MessageButtons::Ok);
    dialog.show();
}

pub struct RemoteDebuggerBackend {
    stream: RefCell<Option<TcpStream>>,
    movies: Vec<Arc<SwfMovie>>,
    path: PathBuf,

    properties: DebuggerProperties,
    squelch: bool,

    packet_kind: Option<ClientMessageKind>,
    data: BytesMut,
}

impl RemoteDebuggerBackend {
    pub fn new(file_url: PathBuf) -> Self {
        Self {
            stream: RefCell::new(None),
            movies: Vec::new(),
            path: file_url,
            properties: DebuggerProperties::default(),
            squelch: false,
            packet_kind: None,
            data: BytesMut::new(),
        }
    }

    fn build(&self, kind: ServerMessageKind) -> DebugSender {
        DebugSender {
            builder: DebugBuilder::new(kind),
            stream: &self.stream,
        }
    }

    fn read_header(&mut self) -> Option<(u32, ClientMessageKind)> {
        if let Some(stream) = self.stream.borrow_mut().as_mut() {
            let mut buf = [0; 8];

            stream.read_exact(&mut buf).ok()?;
            let length = u32::from_le_bytes(buf[..4].try_into().unwrap());
            let message_kind =
                ClientMessageKind::from_u32(u32::from_le_bytes(buf[4..].try_into().unwrap()))?;

            Some((length, message_kind))
        } else {
            None
        }
    }

    fn execute(&mut self, kind: ClientMessageKind) -> Option<bool> {
        match kind {
            ClientMessageKind::SetDebugOption => self.set_debug_option()?,
            ClientMessageKind::GetDebugOption => self.get_debug_option()?,
            ClientMessageKind::SetSquelch => self.set_squelch()?,
            ClientMessageKind::GetInfo => self.get_info()?,
            ClientMessageKind::GetContent => self.get_content()?,
            ClientMessageKind::GetDebugContent => self.get_debug_content()?,
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
        self.build(ServerMessageKind::Squelch)
            .arg(self.squelch as u32)
            .send();
        Some(())
    }

    fn get_content(&mut self) -> Option<()> {
        // TODO: We should be sending all of the SWF's content here.
        // JPEX does not require this, so we'll send it empty for now.
        self.build(ServerMessageKind::SwfImage).send();
        Some(())
    }

    fn get_debug_content(&mut self) -> Option<()> {
        // TODO: When SWD's are supported, this should return SWD content.
        self.build(ServerMessageKind::SwfImage).send();
        Some(())
    }

    fn get_info(&mut self) -> Option<()> {
        let mut builder = self.build(ServerMessageKind::SwfInfo);
        builder.add(self.movies.len() as u16);
        for (i, movie) in self.movies.iter().enumerate() {
            builder.add(i as u32);
            builder.add(Arc::as_ptr(movie) as usize);
            builder.add(false);
            builder.add(0u8);
            builder.add(0u16);
            builder.add(movie.uncompressed_len() as u32);
            builder.add(0u32);
            // TODO: Get script count.
            builder.add(0u32);
            // Offset count
            builder.add(0u32);
            // Breakpoint count
            builder.add(0u32);
            // Port
            builder.add(0u32);
            // Path (empty for now)
            builder.add("");
            // Url (empty for now)
            builder.add("");
            // Host (empty for now)
            builder.add("");
        }
        builder.send();
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
        self.build(ServerMessageKind::DebuggerOption)
            .arg(&*prop)
            .arg(&*val)
            .send();
        Some(())
    }

    fn set_debug_option(&mut self) -> Option<()> {
        let prop = self.read_string()?;
        match &*prop {
            b"disable_script_stuck_dialog" => {
                self.properties.disable_script_stuck_dialog = self.read_switch()?;
                self.build(ServerMessageKind::DebuggerOption)
                    .arg("disable_script_stuck_dialog")
                    .arg(&*self.properties.disable_script_stuck_dialog.to_string())
                    .send();
            }
            b"disable_script_stuck" => {
                self.properties.disable_script_stuck = self.read_switch()?;
                self.build(ServerMessageKind::DebuggerOption)
                    .arg("disable_script_stuck")
                    .arg(&*self.properties.disable_script_stuck.to_string())
                    .send();
            }
            b"break_on_fault" => {
                self.properties.break_on_fault = self.read_switch()?;
                self.build(ServerMessageKind::DebuggerOption)
                    .arg("break_on_fault")
                    .arg(&*self.properties.break_on_fault.to_string())
                    .send();
            }
            b"enumerate_override" => {
                self.properties.enumerate_override = self.read_switch()?;
                self.build(ServerMessageKind::DebuggerOption)
                    .arg("enumerate_override")
                    .arg(&*self.properties.enumerate_override.to_string())
                    .send();
            }
            b"notify_on_failure" => {
                self.properties.notify_on_failure = self.read_switch()?;
                self.build(ServerMessageKind::DebuggerOption)
                    .arg("notify_on_failure")
                    .arg(&*self.properties.notify_on_failure.to_string())
                    .send();
            }
            b"invoke_setters" => {
                self.properties.invoke_setters = self.read_switch()?;
                self.build(ServerMessageKind::DebuggerOption)
                    .arg("invoke_setters")
                    .arg(&*self.properties.invoke_setters.to_string())
                    .send();
            }
            b"wide_line_player" => {
                self.properties.wide_line_player = self.read_switch()?;
                self.build(ServerMessageKind::DebuggerOption)
                    .arg("wide_line_player")
                    .arg(&*self.properties.wide_line_player.to_string())
                    .send();
            }
            b"wide_line_debugger" => {
                self.properties.wide_line_debugger = self.read_switch()?;
                self.build(ServerMessageKind::DebuggerOption)
                    .arg("wide_line_debugger")
                    .arg(&*self.properties.wide_line_debugger.to_string())
                    .send();
            }
            b"swf_load_messages" => {
                self.properties.swf_load_messages = self.read_switch()?;
                self.build(ServerMessageKind::DebuggerOption)
                    .arg("swf_load_messages")
                    .arg(&*self.properties.swf_load_messages.to_string())
                    .send();
            }
            b"getter_timeout" => {
                let prop = self.read_string()?;
                let value = std::str::from_utf8(&prop).ok()?;
                self.properties.getter_timeout = value.parse().ok()?;
                self.build(ServerMessageKind::DebuggerOption)
                    .arg("getter_timeout")
                    .arg(value)
                    .send();
            }
            b"setter_timeout" => {
                let prop = self.read_string()?;
                let value = std::str::from_utf8(&prop).ok()?;
                self.properties.setter_timeout = value.parse().ok()?;
                self.build(ServerMessageKind::DebuggerOption)
                    .arg("setter_timeout")
                    .arg(value)
                    .send();
            }
            _ => (),
        }
        Some(())
    }
}

impl DebuggerBackend for RemoteDebuggerBackend {
    fn tick(&mut self) -> Option<bool> {
        let mut should_continue = false;
        if let Some(kind) = self.packet_kind {
            if let Some(stream) = self.stream.borrow_mut().as_mut() {
                stream.read_exact(&mut self.data).ok()?;
            }

            self.packet_kind = None;
            should_continue = self.execute(kind).unwrap_or(false);
            self.data.clear();
        } else {
            let (length, kind) = self.read_header()?;
            self.data.resize(length as usize, 0);
            self.packet_kind = Some(kind);
        }
        Some(should_continue)
    }

    fn connect(&mut self, password: &str, port: u16) -> bool {
        if let Ok(stream) = TcpStream::connect(("127.0.0.1", port)) {
            stream
                .set_nonblocking(true)
                .expect("failed to set debug stream as nonblocking");
            *self.stream.borrow_mut() = Some(stream);

            self.build(ServerMessageKind::SetVersion)
                .arg(0x0fu32)
                .send();
            self.build(ServerMessageKind::MovieAttribute)
                .arg("movie")
                .arg(self.path.as_os_str())
                .send();
            self.build(ServerMessageKind::MovieAttribute)
                .arg("password")
                .arg(password)
                .send();
            true
        } else {
            false
        }
    }

    fn add_movie(&mut self, movie: Arc<SwfMovie>) {
        self.movies.push(movie)
    }
}
