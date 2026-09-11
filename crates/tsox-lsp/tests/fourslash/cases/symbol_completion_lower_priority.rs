use tsox_lsp::fourslash::{self, Session};


#[test]
fn symbol_completion_lower_priority() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"declare const Symbol: (s: string) => symbol;
const mySymbol = Symbol("test");
interface TestInterface { 
    [mySymbol]: string;
    normalProperty: number;
}
const obj: TestInterface = {} as any;
obj./*completions*/"#;
    let mut s = Session::new_for_test("symbolCompletionLowerPriority", content);
    // TODO: f.VerifyCompletions(t, "completions", &fourslash.CompletionsExpectedList{
}
