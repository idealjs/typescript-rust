use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_tsx_closing_after_jsx_text() {
    let content = r#"// @Filename: foo.tsx

const a = (
    <div>
        text
               </div>
)
const b = (
    <div>
        text
      twice
               </div>
)
"#;
    let mut s = Session::new_for_test("formatTsxClosingAfterJsxText", content);
    // TODO: f.FormatDocument(t, "")
    fourslash::verify_current_file_content(&mut s, r#"
const a = (
    <div>
        text
    </div>
)
const b = (
    <div>
        text
        twice
    </div>
)
"#);
}
