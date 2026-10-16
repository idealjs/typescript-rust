use tsox_lsp::fourslash::Session;


#[test]
fn tsx_find_all_references6() {
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
let opt = <Opt /*1*/wrong />;"#;
    let _s = Session::new_for_test("tsxFindAllReferences6", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1")
}
