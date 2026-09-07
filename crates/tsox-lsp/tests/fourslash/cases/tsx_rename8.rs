use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineRename"]
#[test]
fn tsx_rename8() {
    let content = r#"//@Filename: file.tsx
// @jsx: preserve
// @noLib: true
declare namespace JSX {
    interface Element { }
    interface IntrinsicElements {
    }
    interface ElementAttributesProperty { props; }
}
interface OptionPropBag {
    propx: number
    propString: string
    optional?: boolean
}
declare function Opt(attributes: OptionPropBag): JSX.Element;
let opt = <Opt />;
let opt1 = <Opt propx={100} propString />;
let opt2 = <Opt propx={100} optional/>;
let opt3 = <Opt [|wrong|] />;
let opt4 = <Opt propx={100} propString="hi" />;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/)
}
