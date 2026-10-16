use tsox_lsp::fourslash::Session;


#[test]
fn tsx_find_all_references2() {
    let content = r#"//@Filename: file.tsx
declare namespace JSX {
    interface Element { }
    interface IntrinsicElements {
        div: {
            /*1*/name?: string;
            isOpen?: boolean;
        };
        span: { n: string; };
    }
}
var x = <div name="hello" />;"#;
    let _s = Session::new_for_test("tsxFindAllReferences2", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1")
}
