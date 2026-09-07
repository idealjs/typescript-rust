mod events;
mod thread_id;
mod tracer;

pub use events::{Phase, TraceArg, TraceEvent};
pub use tracer::{EventGuard, Tracer};

#[cfg(test)]
mod tests;
