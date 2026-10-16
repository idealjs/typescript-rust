use tsox_lsp::fourslash::{self, Session};


#[test]
fn incremental_parsing_with_js_doc() {
    let content = r#"[|import a from 'a/aaaaaaa/aaaaaaa/aaaaaa/aaaaaaa';
/**/import b from 'b';
import c from 'c';|]
[|/** @internal */|]
export class LanguageIdentifier[| { }|]"#;
    let mut s = Session::new_for_test("incrementalParsingWithJsDoc", content);
    // TODO: f.VerifyOutliningSpans(t)
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.Backspace(t, 1)
    // TODO: f.VerifyOutliningSpans(t)
}
