use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_in_unclosed_object_type_literal_in_signature02() {
    let content = r#"interface I<TString, TNumber> {
    [s: string]: TString;
    [s: number]: TNumber;
}

declare function foo<TString, TNumber>(obj: I<TString, TNumber>): { str: TStr/*1*/"#;
    let mut s = Session::new_for_test("completionListInUnclosedObjectTypeLiteralInSignature02", content);
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    // TODO: }
}
