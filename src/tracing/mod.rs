//! Compiler tracing, ported from Go `internal/tracing` (`tracing.go`).
//!
//! Records Chrome trace-event JSON (begin/end/instant "B"/"E"/"I" events) for
//! parse/bind/check/emit phases. Each event is assigned a thread id derived
//! from its args: a `checkerId` arg maps to a `checker:<id>` thread, while a
//! file-path arg (`path`/`fileName`/…) maps to a stable `file:<path>` thread
//! via an xxh3 hash.
//!
//! NOTE: The `Tracing` session, `Push`/`Instant`, and the deterministic
//! timestamp mode are **not yet implemented** in Rust. (This is distinct from
//! the external `tracing` logging crate, which is currently an unused
//! dependency.) The test stubs below document the Go test data from
//! `internal/tracing/tracing_test.go` and are marked `#[ignore]` until the
//! implementation lands.

#[cfg(test)]
mod tests {
    // Both Go tests build an in-memory VFS (`/trace` dir), call
    // `StartTracing(fsys, "/trace", "", deterministic=true)`, push events,
    // `StopTracing`, then parse `/trace/trace.json` and assert on thread ids.
    //
    // Helpers referenced by the Go tests:
    //   findEvent(events, ph, name, argName, argValue) -> traceEvent
    //   assertThreadName(events, tid, name)
    //   assertDurationEventsAreWellNestedByThread(events)  // B/E stack check

    /// Go `TestConcurrentDurationEventsUseSeparateThreadIDs`.
    ///
    /// Pushes two interleaved `createSourceFile` parse events with `path`
    /// `/a.ts` and `/b.ts` (separateBeginAndEnd=true), then a nested
    /// `checkSourceFile` (checkerId=0, path `/a.ts`) containing a
    /// `getVariancesWorker` (checkerId=0, id=1) event.
    ///
    /// Assertions (deterministic mode):
    /// - `/a.ts` begin/end share a tid; `/b.ts` begin/end share a tid; the two
    ///   tids differ.
    /// - thread_name for `/a.ts` == "file:/a.ts", for `/b.ts` == "file:/b.ts".
    /// - `checkSourceFile` and `getVariancesWorker` share a tid named
    ///   "checker:0".
    /// - all duration (B/E) events are well-nested per thread.
    #[test]
    #[ignore = "tracing::Tracing / Push / StartTracing not yet ported to Rust"]
    fn concurrent_duration_events_use_separate_thread_ids() {
        // events sequence (separateBeginAndEnd = true):
        //   B createSourceFile  {path: "/a.ts"}
        //   B createSourceFile  {path: "/b.ts"}
        //   E createSourceFile  {path: "/a.ts"}
        //   E createSourceFile  {path: "/b.ts"}
        //   B checkSourceFile   {checkerId: 0, path: "/a.ts"}
        //   B getVariancesWorker{checkerId: 0, id: 1}
        //   E getVariancesWorker{checkerId: 0, id: 1}
        //   E checkSourceFile   {checkerId: 0, path: "/a.ts"}
    }

    /// Go `TestThreadIDsAreStableAcrossFirstSeenOrder`.
    ///
    /// Runs the same two-file parse trace twice with the paths in opposite
    /// order (`["/a.ts","/b.ts"]` then `["/b.ts","/a.ts"]`) and asserts the
    /// resulting `{path -> tid}` maps are identical. This verifies that file
    /// thread ids are derived from a stable hash of the path rather than from
    /// first-seen allocation order.
    #[test]
    #[ignore = "tracing::Tracing / Push / StartTracing not yet ported to Rust"]
    fn thread_ids_are_stable_across_first_seen_order() {
        // first  = trace_thread_ids_for_paths(["/a.ts", "/b.ts"])
        // second = trace_thread_ids_for_paths(["/b.ts", "/a.ts"])
        // assert_eq!(first, second)
    }
}
