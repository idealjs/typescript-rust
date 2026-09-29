use tsox_lsp::fourslash::{self, Session};


#[test]
fn no_errors_after_completions_request_within_generic_function2() {
    let content = r#"// @strict: true

// repro from #50818#issuecomment-1278324638

declare function func<T extends { foo: 1 }>(arg: T): void;
func({ foo: 1, bar/*1*/: 1 });"#;
    let mut s = Session::new_for_test("noErrorsAfterCompletionsRequestWithinGenericFunction2", content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::insert(&mut s, "2");
    fourslash::verify_completions_empty_at(&mut s, None);
    fourslash::verify_no_errors(&mut s, );
}
