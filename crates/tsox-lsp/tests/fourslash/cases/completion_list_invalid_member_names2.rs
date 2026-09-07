use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn completion_list_invalid_member_names2() {
    // TODO: t.Skip("Known failing fourslash test")
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
