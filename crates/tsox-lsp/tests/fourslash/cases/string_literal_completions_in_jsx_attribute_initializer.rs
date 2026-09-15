use tsox_lsp::fourslash::Session;


#[test]
fn string_literal_completions_in_jsx_attribute_initializer() {
    let content = r#"// @jsx: preserve
// @filename: /a.tsx
type Props = { a: number } | { b: "somethingelse", c: 0 | 1 };
declare function Foo(args: Props): any

const a1 = <Foo b={"/*1*/"} />
const a2 = <Foo b="/*2*/" />
const a3 = <Foo b="somethingelse"/*3*/ />
const a4 = <Foo b={"somethingelse"} /*4*/ />
const a5 = <Foo b={"somethingelse"} c={0} /*5*/ />"#;
    let _s = Session::new_for_test("stringLiteralCompletionsInJsxAttributeInitializer", content);
    // TODO: f.VerifyCompletions(t, []string{"1", "2"}, &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, []string{"3", "4"}, &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, []string{"5"}, &fourslash.CompletionsExpectedList{
}
