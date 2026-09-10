use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn tsx_completion12() {
    let content = r#"//@Filename: file.tsx
// @jsx: preserve
// @noLib: true
declare module JSX {
    interface Element { }
    interface IntrinsicElements {
    }
    interface ElementAttributesProperty { props; }
}
interface OptionPropBag {
    propx: number
    propString: "hell"
    optional?: boolean
}
declare function Opt(attributes: OptionPropBag): JSX.Element;
let opt = <Opt /*1*/ />;
let opt1 = <Opt [|prop|]/*2*/ />;
let opt2 = <Opt propx={100} /*3*/ />;
let opt3 = <Opt propx={100} optional /*4*/ />;
let opt4 = <Opt wrong /*5*/ />;"#;
    let mut s = Session::new_for_test("tsxCompletion12", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"1", "5"}, &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
    fourslash::verify_completions_exact_at(&mut s, Some("4"), &["propString"]);
}
