use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_object_literal_method1() {
    let content = r#"// @newline: LF
// @Filename: a.ts
interface IFoo {
    bar(x: number): void;
}

const obj: IFoo = {
    /*a*/
}
type Foo = {
    bar(x: number): void;
    foo: (x: string) => string;
}

const f: Foo = {
    /*b*/
}

interface Overload {
    buzz(a: number): number;
    buzz(a: string): string;
}
const o: Overload = {
    /*c*/
}
interface Prop {
    "space bar"(): string;
}
const p: Prop = {
    /*d*/
}"#;
    let mut s = Session::new_with_capabilities(content, None);
    // TODO: f.VerifyCompletions(t, "a", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "b", &fourslash.CompletionsExpectedList{
    // TODO: {
    // TODO: f.VerifyCompletions(t, "d", &fourslash.CompletionsExpectedList{
    // TODO: {
    // TODO: {
}
