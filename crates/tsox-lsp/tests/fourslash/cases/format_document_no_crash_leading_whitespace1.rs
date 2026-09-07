use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: const content = ' \n'"]
#[test]
fn format_document_no_crash_leading_whitespace1() {
    // TODO: const content = " \n"
    let mut s = Session::new("");
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
}
