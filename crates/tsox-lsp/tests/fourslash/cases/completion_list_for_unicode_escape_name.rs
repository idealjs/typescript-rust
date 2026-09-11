use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_for_unicode_escape_name() {
    let content = r#"function \u0042 () { /*0*/ }
export default function \u0043 () {}
class \u0041 { /*2*/ }
/*3*/"#;
    let mut s = Session::new_for_test("completionListForUnicodeEscapeName", content);
    fourslash::verify_completions_include_exclude_at(&mut s, Some("0"), &["B"], &[]);
    // TODO: f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    fourslash::verify_completions_include_exclude_at(&mut s, Some("3"), &["B", "A", "C"], &[]);
}
