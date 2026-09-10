use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.Backspace"]
#[test]
fn incremental_parsing_with_js_doc() {
    let content = r#"[|import a from 'a/aaaaaaa/aaaaaaa/aaaaaa/aaaaaaa';
/**/import b from 'b';
import c from 'c';|]
[|/** @internal */|]
export class LanguageIdentifier[| { }|]"#;
    let mut s = Session::new_for_test("incrementalParsingWithJsDoc", content);
    fourslash::unsupported("VerifyOutliningSpans"); // f.VerifyOutliningSpans(t)
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("Backspace"); // f.Backspace(t, 1)
    fourslash::unsupported("VerifyOutliningSpans"); // f.VerifyOutliningSpans(t)
}
