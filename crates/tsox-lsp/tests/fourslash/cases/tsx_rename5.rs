use tsox_lsp::fourslash::{self, Session};


#[test]
fn tsx_rename5() {
    let content = r#"//@Filename: file.tsx
declare namespace JSX {
    interface Element { }
    interface IntrinsicElements {
    }
    interface ElementAttributesProperty { props }
}
class MyClass {
  props: {
    name?: string;
    size?: number;
}

[|var [|{| "contextRangeIndex": 0 |}nn|]: string;|]
var x = <MyClass name={[|nn|]}></MyClass>;"#;
    let mut s = Session::new_for_test("tsxRename5", content);
    // TODO: f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "nn")
    // TODO: }
}
