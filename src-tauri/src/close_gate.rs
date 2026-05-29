use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// When the user confirms quit, the frontend calls [`crate::commands::grant_next_close`], then
/// `WebviewWindow::close()`. The next `CloseRequested` consumes this flag and allows the window
/// to close instead of emitting `diagramme-close-request`.
#[derive(Clone, Default)]
pub struct AllowNextClose(pub Arc<AtomicBool>);

impl AllowNextClose {
    pub fn grant(&self) {
        self.0.store(true, Ordering::SeqCst);
    }

    /// Returns `true` if a close should proceed (flag was set).
    pub fn consume_if_allowed(&self) -> bool {
        self.0.swap(false, Ordering::SeqCst)
    }
}
