use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: }"]
#[test]
fn tsx_find_all_references4() {
    let content = r#"//@Filename: file.tsx
declare namespace JSX {
    interface Element { }
    interface IntrinsicElements {
    }
    interface ElementAttributesProperty { props }
}
/*1*/class /*2*/MyClass {
  props: {
    name?: string;
    size?: number;
}


var x = /*3*/</*4*/MyClass name='hello'><//*5*/MyClass>;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4", "5")
    // TODO: }
}
