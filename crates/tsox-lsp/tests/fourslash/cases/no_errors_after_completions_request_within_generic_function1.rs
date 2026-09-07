use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn no_errors_after_completions_request_within_generic_function1() {
    let content = r#"// @strict: true

declare function func<T extends { foo: 1 }>(arg: T): void;
func({ foo: 1, bar/*1*/: 1 });"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, nil, nil)
    fourslash::verify_no_errors(&mut s);
}
