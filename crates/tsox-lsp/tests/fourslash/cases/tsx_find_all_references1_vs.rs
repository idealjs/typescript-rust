use tsox_lsp::fourslash::Session;


#[test]
fn tsx_find_all_references1_vs() {
    let content = r#"//@Filename: file.tsx
declare namespace JSX {
    interface Element { }
    interface IntrinsicElements {
        /*1*/div: {
            name?: string;
            isOpen?: boolean;
        };
        span: { n: string; };
    }
}
var x = /*2*/</*3*/div />;"#;
    let _s = Session::new_with_capabilities(content, None);
    // TODO: f.VerifyBaselineVSFindAllReferences(t, "1", "2", "3")
}
