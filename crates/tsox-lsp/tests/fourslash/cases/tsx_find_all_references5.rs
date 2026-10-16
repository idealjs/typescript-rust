use tsox_lsp::fourslash::Session;


#[test]
fn tsx_find_all_references5() {
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
/*1*/declare function /*2*/Opt(attributes: OptionPropBag): JSX.Element;
let opt = /*3*/</*4*/Opt />;
let opt1 = /*5*/</*6*/Opt propx={100} propString />;
let opt2 = /*7*/</*8*/Opt propx={100} optional/>;
let opt3 = /*9*/</*10*/Opt wrong />;
let opt4 = /*11*/</*12*/Opt propx={100} propString="hi" />;"#;
    let _s = Session::new_for_test("tsxFindAllReferences5", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12")
}
