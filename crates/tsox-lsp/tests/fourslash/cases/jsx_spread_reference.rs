use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineRenameAtRangesWithText"); // f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "nn")
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "src")
}
