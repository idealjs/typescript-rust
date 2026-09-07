use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: const content = 'type X = {\n\tb}'"]
#[test]
fn format_document_no_crash_short_last_line() {
    // TODO: const content = "type X = {\n\tb}"
    let mut s = Session::new("");
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
}
