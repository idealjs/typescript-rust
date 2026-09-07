use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "completions", &fourslash.CompletionsExpectedList{
}
