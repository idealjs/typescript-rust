use tsox_lsp::fourslash::{self, Session};


#[test]
fn tsx_find_all_references3() {
    let content = r#"//@Filename: file.tsx
declare namespace JSX {
    interface Element { }
    interface IntrinsicElements {
    }
    interface ElementAttributesProperty { props }
}
class MyClass {
  props: {
    /*1*/name?: string;
    size?: number;
}


var x = <MyClass name='hello'/>;"#;
    let mut s = Session::new_for_test("tsxFindAllReferences3", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1")
    // TODO: }
}
