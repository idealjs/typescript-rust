use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_with_deprecated_tag1() {
    let content = r#"// @strict: true
// @filename: /foobar.ts
/** @deprecated */
export function foobar() {}
// @filename: /foo.ts
import { foobar/*4*/ } from "./foobar";

/** @deprecated */
interface Foo {
    /** @deprecated */
    bar(): void
    /** @deprecated */
    prop: number
}
declare const foo: Foo;
declare const foooo: Fo/*1*/;
foo.ba/*2*/;
foo.pro/*3*/;

fooba/*5*/;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "4", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "5", &fourslash.CompletionsExpectedList{
}
