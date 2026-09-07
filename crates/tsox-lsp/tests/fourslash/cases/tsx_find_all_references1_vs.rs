use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineVSFindAllReferences"]
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
    let mut s = Session::new_with_capabilities(content, None);
    fourslash::unsupported("VerifyBaselineVSFindAllReferences"); // f.VerifyBaselineVSFindAllReferences(t, "1", "2", "3")
}
