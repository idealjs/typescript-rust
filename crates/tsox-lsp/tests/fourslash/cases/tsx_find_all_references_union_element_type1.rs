use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn tsx_find_all_references_union_element_type1() {
    let content = r#"//@Filename: file.tsx
// @jsx: preserve
// @noLib: true
declare namespace JSX {
    interface Element { }
    interface IntrinsicElements {
    }
    interface ElementAttributesProperty { props; }
}
function SFC1(prop: { x: number }) {
    return <div>hello </div>;
};
function SFC2(prop: { x: boolean }) {
    return <h1>World </h1>;
}
/*1*/var /*2*/SFCComp = SFC1 || SFC2;
/*3*/</*4*/SFCComp x={ "hi" } />"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4")
}
