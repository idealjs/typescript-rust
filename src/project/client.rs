//! LSP client interface (1:1 port of Go's `internal/project/client.go`).
//!
//! The `Client` trait abstracts the LSP client connection so the session
//! can send requests/notifications without depending on transport details.

#![allow(dead_code)]


use crate::diagnostics;
use crate::lsp::lsproto;

use super::watch::WatcherID;

/// Client represents the LSP client the server communicates with.
///
/// Go: `type Client interface { ... }`
pub trait Client: Send + Sync {
    fn watch_files(
        &self,
        id: &WatcherID,
        watchers: &[lsproto::FileSystemWatcher],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    fn unwatch_files(&self, id: &WatcherID)
    -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    fn refresh_diagnostics(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    fn publish_diagnostics(
        &self,
        params: &lsproto::PublishDiagnosticsParams,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    fn refresh_inlay_hints(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    fn refresh_code_lens(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    fn progress_start(&self, message: &diagnostics::Message, args: &[Box<dyn std::fmt::Debug>]);

    fn progress_finish(&self, message: &diagnostics::Message, args: &[Box<dyn std::fmt::Debug>]);

    fn send_telemetry(
        &self,
        telemetry: &lsproto::TelemetryEvent,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    fn is_active(&self) -> bool;
}

/// A no-op Client implementation used when no real client is connected.
pub struct NopClient;

impl Client for NopClient {
    fn watch_files(
        &self,
        _id: &WatcherID,
        _watchers: &[lsproto::FileSystemWatcher],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }

    fn unwatch_files(
        &self,
        _id: &WatcherID,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }

    fn refresh_diagnostics(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }

    fn publish_diagnostics(
        &self,
        _params: &lsproto::PublishDiagnosticsParams,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }

    fn refresh_inlay_hints(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }

    fn refresh_code_lens(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }

    fn progress_start(&self, _message: &diagnostics::Message, _args: &[Box<dyn std::fmt::Debug>]) {}

    fn progress_finish(&self, _message: &diagnostics::Message, _args: &[Box<dyn std::fmt::Debug>]) {
    }

    fn send_telemetry(
        &self,
        _telemetry: &lsproto::TelemetryEvent,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }

    fn is_active(&self) -> bool {
        false
    }
}
