//! Construct trace module for tracking which constructs are parsed during fixture parsing.
//!
//! ensuring byte-identical production behavior.

#![allow(dead_code)]

use std::cell::RefCell;
use std::collections::HashSet;

/// Trace data for a single fixture parse
#[derive(Debug, Clone, Default)]
pub struct FixtureTrace {
    /// The fixture name being parsed
    pub fixture: String,
    /// Set of construct IDs that were hit during parsing
    pub constructs: HashSet<&'static str>,
}

// Thread-local storage for the current trace
thread_local! {
    static CURRENT: RefCell<Option<FixtureTrace>> = const { RefCell::new(None) };
}

/// Begin tracing for a fixture
pub fn begin(fixture: &str) {
    CURRENT.with(|cell| {
        *cell.borrow_mut() = Some(FixtureTrace {
            fixture: fixture.to_string(),
            constructs: HashSet::new(),
        });
    });
}

/// End tracing and return the trace data
pub fn end() -> FixtureTrace {
    CURRENT.with(|cell| {
        cell.borrow_mut()
            .take()
            .expect("Trace not initialized - call begin() first")
    })
}

/// Record a construct hit
pub fn hit(construct: &'static str) {
    CURRENT.with(|cell| {
        if let Some(trace) = cell.borrow_mut().as_mut() {
            trace.constructs.insert(construct);
        }
    });
}

/// Get the current trace without consuming it (for debugging)
pub fn with_trace<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&FixtureTrace) -> R,
{
    CURRENT.with(|cell| cell.borrow().as_ref().map(f))
}