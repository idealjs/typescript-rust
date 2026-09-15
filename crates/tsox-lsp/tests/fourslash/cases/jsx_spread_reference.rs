use tsox_lsp::fourslash::Session;


#[test]
fn jsx_spread_reference() {
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
}

[|var [|/*dst*/{| "contextRangeIndex": 0 |}nn|]: {name?: string; size?: number};|]
var x = <MyClass {...[|n/*src*/n|]}></MyClass>;"#;
    let _s = Session::new_for_test("jsxSpreadReference", content);
    // TODO: f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "nn")
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "src")
}
