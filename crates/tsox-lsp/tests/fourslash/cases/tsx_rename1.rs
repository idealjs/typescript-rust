use tsox_lsp::fourslash::{self, Session};


#[test]
fn tsx_rename1() {
    let content = r#"//@Filename: file.tsx
declare namespace JSX {
    interface Element { }
    interface IntrinsicElements {
        [|[|{| "contextRangeIndex": 0 |}div|]: {
            name?: string;
            isOpen?: boolean;
        };|]
        span: { n: string; };
    }
}
var x = [|<[|{| "contextRangeIndex": 2 |}div|] />|];"#;
    let mut s = Session::new_for_test("tsxRename1", content);
    // TODO: f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "div")
}
