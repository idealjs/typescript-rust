#![allow(dead_code)]

use crate::lsp::lsproto;

use super::watch::WatcherID;

pub trait Client: Send + Sync {
    fn watch_files(
        &self,
        id: &WatcherID,
        watchers: &[lsproto::FileSystemWatcher],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    fn unwatch_files(&self, id: &WatcherID)
    -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    fn register_content_mapper_extensions(&self, extensions: &[String]) -> Result<(), String>;

    fn refresh_diagnostics(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    fn publish_diagnostics(
        &self,
        params: &lsproto::PublishDiagnosticsParams,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    fn refresh_inlay_hints(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    fn refresh_code_lens(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    fn progress_start(
        &self,
        message: &tsox_core::diagnostics::Message,
        args: &[Box<dyn std::fmt::Debug>],
    );

    fn progress_finish(
        &self,
        message: &tsox_core::diagnostics::Message,
        args: &[Box<dyn std::fmt::Debug>],
    );

    fn send_telemetry(
        &self,
        telemetry: &lsproto::TelemetryEvent,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    fn get_locale(&self) -> tsox_core::locale::Locale;

    fn is_active(&self) -> bool;
}

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

    fn register_content_mapper_extensions(&self, _extensions: &[String]) -> Result<(), String> {
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

    fn progress_start(
        &self,
        _message: &tsox_core::diagnostics::Message,
        _args: &[Box<dyn std::fmt::Debug>],
    ) {
    }

    fn progress_finish(
        &self,
        _message: &tsox_core::diagnostics::Message,
        _args: &[Box<dyn std::fmt::Debug>],
    ) {
    }

    fn send_telemetry(
        &self,
        _telemetry: &lsproto::TelemetryEvent,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }

    fn get_locale(&self) -> tsox_core::locale::Locale {
        tsox_core::locale::Locale::default()
    }

    fn is_active(&self) -> bool {
        false
    }
}
