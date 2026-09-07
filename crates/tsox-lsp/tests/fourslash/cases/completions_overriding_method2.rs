use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_overriding_method2() {
    let content = r#"// @newline: LF
// @Filename: a.ts
interface DollarSign {
    "$usd"(a: number): number;
    $cad(b: number): number;
    cla$$y(c: number): number;
    isDollarAmountString(s: string): s is ` + "`" + `$${number}` + "`" + `
}
class USD implements DollarSign {
    /*a*/
}"#;
    let mut s = Session::new_with_capabilities(content, None);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "a", &fourslash.CompletionsExpectedList{
}
