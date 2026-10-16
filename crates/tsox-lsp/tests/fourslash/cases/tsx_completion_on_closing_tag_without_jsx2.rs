use tsox_lsp::fourslash::{self, Session};


#[test]
fn tsx_completion_on_closing_tag_without_jsx2() {
    let content = r#"//@Filename: file.tsx
var x1 = <div>
   <h1> Hello world </ /*2*/>
   </ /*1*/>"#;
    let mut s = Session::new_for_test("tsxCompletionOnClosingTagWithoutJSX2", content);
    fourslash::verify_completions_exact_at(&mut s, Some("1"), &["div"]);
    fourslash::verify_completions_exact_at(&mut s, Some("2"), &["h1"]);
}
