use tsox_lsp::fourslash::Session;


#[test]
fn tsx_rename2() {
    let content = r#"//@Filename: file.tsx
declare namespace JSX {
    interface Element { }
    interface IntrinsicElements {
        div: {
            [|[|{| "contextRangeIndex": 0 |}name|]?: string;|]
            isOpen?: boolean;
        };
        span: { n: string; };
    }
}
var x = <div [|[|{| "contextRangeIndex": 2 |}name|]="hello"|] />;"#;
    let _s = Session::new_for_test("tsxRename2", content);
    // TODO: f.VerifyBaselineRenameAtRangesWithText(t, nil /*preferences*/, "name")
}
