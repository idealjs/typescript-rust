use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_generic_indexed_access1() {
    let content = r#"interface Sample {
  addBook: { name: string, year: number }
}

export declare function testIt<T>(method: T[keyof T]): any
testIt<Sample>({ /**/ });"#;
    let mut s = Session::new_for_test("completionsGenericIndexedAccess1", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
