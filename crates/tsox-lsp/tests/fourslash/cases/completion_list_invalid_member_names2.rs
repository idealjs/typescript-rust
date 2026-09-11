use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn completion_list_invalid_member_names2() {
    let content = r#"// @lib: es5
declare var Symbol: SymbolConstructor;
interface SymbolConstructor {
    readonly hasInstance: symbol;
}
interface Function {
    [Symbol.hasInstance](value: any): boolean;
}
interface SomeInterface {
    (value: number): any;
}
var _ : SomeInterface;
_./**/"#;
    let mut s = Session::new_for_test("completionListInvalidMemberNames2", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
