use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: const content = ' \n;'"]
#[test]
fn format_document_no_crash_leading_whitespace2() {
    // TODO: const content = " \n;"
    let mut s = Session::new_for_test("formatDocumentNoCrashLeadingWhitespace2", "");
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
}
