use std::{
    fmt,
    panic::{self, UnwindSafe},
};

#[derive(Debug, Clone, Copy)]
pub enum Cancelled {
    /// The query was operating on revision R, but there is a pending write to move to revision R+1.
    #[non_exhaustive]
    PendingWrite,

    /// The query was blocked on another thread, and that thread panicked.
    #[non_exhaustive]
    PropagatedPanic,
}

impl Cancelled {
    fn throw(self) -> ! {
        // We use resume and not panic here to avoid running the panic
        // hook (that is, to avoid collecting and printing backtrace).
        std::panic::resume_unwind(Box::new(self));
    }

    /// Runs `f`, and catches any salsa cancellation.
    pub fn catch<F, T>(f: F) -> Result<T, Cancelled>
    where
        F: FnOnce() -> T + UnwindSafe,
    {
        match panic::catch_unwind(f) {
            Ok(t) => Ok(t),
            Err(payload) => match payload.downcast() {
                Ok(cancelled) => Err(*cancelled),
                Err(payload) => panic::resume_unwind(payload),
            },
        }
    }
}

impl std::fmt::Display for Cancelled {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let why = match self {
            Cancelled::PendingWrite => "pending write",
            Cancelled::PropagatedPanic => "propagated panic",
        };
        f.write_str("cancelled because of ")?;
        f.write_str(why)
    }
}

impl std::error::Error for Cancelled {}
#[derive(Debug)]
pub(crate) struct LspError {
    pub(crate) code: i32,
    pub(crate) message: String,
}

impl LspError {
    pub(crate) fn new(code: i32, message: String) -> LspError {
        LspError { code, message }
    }
}

impl fmt::Display for LspError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Language Server request failed with {}. ({})",
            self.code, self.message
        )
    }
}

impl std::error::Error for LspError {}
