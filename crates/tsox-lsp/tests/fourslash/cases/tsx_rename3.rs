use tsox_lsp::fourslash::Session;


#[test]
fn tsx_rename3() {
    let content = r#"//@Filename: file.tsx
declare namespace JSX {
    interface Element { }
    interface IntrinsicElements {
    }
    interface ElementAttributesProperty { props }
}
class MyClass {
  props: {
    [|[|{| "contextRangeIndex": 0 |}name|]?: string;|]
    size?: number;
}


var x = <MyClass [|[|{| "contextRangeIndex": 2 |}name|]='hello'|]/>;"#;
    let _s = Session::new_for_test("tsxRename3", content);
    // TODO: f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "name")
    // TODO: }
}
