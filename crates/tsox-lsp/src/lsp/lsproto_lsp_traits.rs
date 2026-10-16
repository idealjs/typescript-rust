use crate::lsp::lsproto_lsp_basic::Location;
use crate::lsp::lsproto_lsp_basic::Position;
use crate::lsp::lsproto_lsp_uri::DocumentUri;

pub trait HasTextDocumentUri {
    fn text_document_uri(&self) -> &DocumentUri;
}

pub trait HasTextDocumentPosition: HasTextDocumentUri {
    fn text_document_position(&self) -> &Position;
}

pub trait HasLocations {
    fn get_locations(&self) -> &Vec<Location>;
}

pub trait HasLocation {
    fn get_location(&self) -> &Location;
}
