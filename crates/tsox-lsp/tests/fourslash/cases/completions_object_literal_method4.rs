use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_object_literal_method4() {
    let content = r#"// @newline: LF
// @Filename: a.ts
interface IFoo {
    bar(this: IFoo): void;
}
const obj: IFoo = {
    /*1*/
}"#;
    let mut s = Session::new_with_capabilities(content, None);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
