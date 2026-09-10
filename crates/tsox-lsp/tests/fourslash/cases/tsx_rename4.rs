use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineRenameAtRangesWithText"]
#[test]
fn tsx_rename4() {
    let content = r#"// @jsx: preserve
//@Filename: file.tsx
declare namespace JSX {
    interface Element {}
    interface IntrinsicElements {
        div: {};
    }
}
[|class [|{| "contextRangeIndex": 0 |}MyClass|] {}|]

[|<[|{| "contextRangeIndex": 2 |}MyClass|]></[|{| "contextRangeIndex": 2 |}MyClass|]>|];
[|<[|{| "contextRangeIndex": 5 |}MyClass|]/>|];

[|<[|{| "contextRangeIndex": 7 |}div|]> </[|{| "contextRangeIndex": 7 |}div|]>|]"#;
    let mut s = Session::new_for_test("tsxRename4", content);
    fourslash::verify_no_errors(&mut s, );
    fourslash::unsupported("VerifyBaselineRenameAtRangesWithText"); // f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "MyClass", "div")
}
