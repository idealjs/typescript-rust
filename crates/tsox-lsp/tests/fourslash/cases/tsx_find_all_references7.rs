use tsox_lsp::fourslash::{self, Session};


#[test]
fn tsx_find_all_references7() {
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
    /*1*/propx: number
    propString: string
    optional?: boolean
}
declare function Opt(attributes: OptionPropBag): JSX.Element;
let opt = <Opt />;
let opt1 = <Opt propx={100} propString />;
let opt2 = <Opt propx={100} optional/>;
let opt3 = <Opt wrong />;"#;
    let mut s = Session::new_for_test("tsxFindAllReferences7", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1")
}
