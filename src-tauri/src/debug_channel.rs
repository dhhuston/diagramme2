//! Thread-local debug log buffer. IPC command handlers call `drain()` to collect
//! buffered messages and emit them to the frontend via a Tauri event.

use std::cell::RefCell;

#[derive(Clone, serde::Serialize)]
pub struct DebugMsg {
    pub level: &'static str, // "info" | "warn" | "error"
    pub msg: String,
}

const CAP: usize = 200;

thread_local! {
    static BUFFER: RefCell<Vec<DebugMsg>> = const { RefCell::new(Vec::new()) };
}

fn push_level(level: &'static str, msg: impl Into<String>) {
    BUFFER.with(|b| {
        let mut buf = b.borrow_mut();
        if buf.len() < CAP {
            buf.push(DebugMsg {
                level,
                msg: msg.into(),
            });
        }
    });
}

#[allow(dead_code)]
pub fn push(msg: impl Into<String>) {
    push_level("info", msg);
}

#[allow(dead_code)]
pub fn push_warn(msg: impl Into<String>) {
    push_level("warn", msg);
}

#[allow(dead_code)]
pub fn push_error(msg: impl Into<String>) {
    push_level("error", msg);
}

/// Take all buffered messages, clearing the buffer.
pub fn drain() -> Vec<DebugMsg> {
    BUFFER.with(|b| b.borrow_mut().drain(..).collect())
}
