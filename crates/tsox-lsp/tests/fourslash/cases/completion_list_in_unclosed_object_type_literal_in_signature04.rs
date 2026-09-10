use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: }"]
#[test]
fn completion_list_in_unclosed_object_type_literal_in_signature04() {
    let content = r#"interface I<TString, TNumber> {
    [s: string]: TString;
    [s: number]: TNumber;
}

declare function foo<TString, TNumber>(obj: I<TString, TNumber>): { /*1*/"#;
    let mut s = Session::new_for_test("completionListInUnclosedObjectTypeLiteralInSignature04", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    // TODO: }
}
