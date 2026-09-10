use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn tsx_go_to_definition_union_element_type1() {
    let content = r#"//@Filename: file.tsx
// @jsx: preserve
// @noLib: true
declare namespace JSX {
    interface Element { }
    interface IntrinsicElements {
    }
    interface ElementAttributesProperty { props; }
}
function /*pt1*/SFC1(prop: { x: number }) {
    return <div>hello </div>;
};
function SFC2(prop: { x: boolean }) {
    return <h1>World </h1>;
}
var /*def*/SFCComp = SFC1 || SFC2;
<[|SFC/*one*/Comp|] x />"#;
    let mut s = Session::new_for_test("tsxGoToDefinitionUnionElementType1", content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "one")
}
