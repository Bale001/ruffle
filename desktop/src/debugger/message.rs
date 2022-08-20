use num_derive::{FromPrimitive, ToPrimitive};

/// Represents a kind of message sent from the debugger client
#[allow(dead_code)]
#[derive(FromPrimitive, Debug, Clone, Copy)]
pub enum ClientMessageKind {
    ZoomIn = 0x00,
    ZoomOut = 0x01,
    ZoomComplete = 0x02,
    Home = 0x03,
    SetQuality = 0x04,
    Play = 0x05,
    Loop = 0x06,
    Rewind = 0x07,
    Forward = 0x08,
    Back = 0x09,
    Print = 0x0A,
    SetField = 0x0B,
    SetProperty = 0x0C,
    TerminateSession = 0x0D,
    RequestProps = 0x0E,
    Continue = 0x0F,
    Suspend = 0x10,
    SetBreak = 0x11,
    ClearBreak = 0x12,
    ClearAllBreak = 0x13,
    StepOver = 0x14,
    StepInto = 0x15,
    StepOut = 0x16,
    ProcessedTag = 0x17,
    SetSquelch = 0x18,
    GetField = 0x19,
    GetFuncName = 0x1A,
    GetDebugOption = 0x1B,
    SetDebugOption = 0x1C,
    AddWatch = 0x1D,
    RemoveWatch = 0x1E,
    StepContinue = 0x1F,
    GetContent = 0x20,
    GetDebugContent = 0x21,
    GetFieldGetterInvoker = 0x22,
    GetSuspendReason = 0x23,
    GetActions = 0x24,
    SetActions = 0x25,
    GetInfo = 0x26,
    GetConstantPool = 0x27,
    GetFuncInfo = 0x28,
    AddWatch2 = 0x31,
    RemoveWatch2 = 0x32,
}

#[allow(dead_code)]
#[derive(ToPrimitive, Debug, Clone, Copy)]
pub enum ServerMessageKind {
    SetVersion = 0x1A,
    Continue = 0x11,
    Squelch = 0x1D,
    DebuggerOption = 0x20,
    MovieAttribute = 0x0C,
    NumSwdEntries = 0x14,
    SwfInfo = 0x2a,
    SwfImage = 0x22,
    SwdImage = 0x23,
    SuspendReason = 0x28,
}

#[derive(Debug, Copy, Clone)]
#[repr(u16)]
#[allow(dead_code)]
pub enum SuspendReason {
    Unknown = 0,
    Breakpoint = 1,
    Watch = 2,
    Fault = 3,
    StopRequest = 4,
    Step = 5,
    Halt = 6,
    ScriptLoaded = 7,
}
