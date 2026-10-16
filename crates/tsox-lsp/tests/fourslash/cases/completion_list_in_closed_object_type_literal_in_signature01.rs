use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_in_closed_object_type_literal_in_signature01() {
    let content = r#"interface I<TString, TNumber> {
    [s: string]: TString;
    [s: number]: TNumber;
}

declare function foo<TString, TNumber>(obj: I<TString, TNumber>): { str: T/*1*/ }"#;
    let mut s = Session::new_for_test("completionListInClosedObjectTypeLiteralInSignature01", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
