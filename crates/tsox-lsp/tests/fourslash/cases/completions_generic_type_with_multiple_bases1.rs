use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_generic_type_with_multiple_bases1() {
    let content = r#"export interface iBaseScope {
    watch: () => void;
}
export interface iMover {
    moveUp: () => void;
}
export interface iScope<TModel> extends iBaseScope, iMover {
    family: TModel;
}
var x: iScope<number>;
x./**/"#;
    let mut s = Session::new_for_test("completionsGenericTypeWithMultipleBases1", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
