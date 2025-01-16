use parking_lot::Mutex;
use std::fmt;
use std::sync::Arc;

struct Inner {
    callback: Option<Box<dyn FnOnce() + Send + 'static>>,
}

impl Drop for Inner {
    fn drop(&mut self) {
        if let Some(callback) = self.callback.take() {
            callback();
        }
    }
}

/// Handler ID keeps event handler in place, once dropped handler will be removed automatically.
///
/// [`HandlerId::detach()`] can be used if it is not desirable for handler to be removed
/// automatically.
#[must_use = "Handler will be unregistered immediately if not used"]
#[derive(Clone)]
pub struct HandlerId {
    inner: Arc<Mutex<Inner>>,
}

impl fmt::Debug for HandlerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HandlerId").finish()
    }
}

impl HandlerId {
    pub(crate) fn new<F>(f: F) -> HandlerId
    where
        F: FnOnce() + Send + 'static,
    {
        let inner = Arc::new(Mutex::new(Inner {
            callback: Some(Box::new(f)),
        }));

        HandlerId { inner }
    }

    /// Consumes [`HandlerId`] and prevents handler from being removed automatically.
    pub fn detach(&self) {
        // Remove callback such that it is not called in drop implementation
        self.inner.lock().callback.take();
    }
}
