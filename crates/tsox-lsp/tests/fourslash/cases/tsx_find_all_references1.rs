use tsox_lsp::fourslash::{self, Session};


#[test]
fn tsx_find_all_references1() {
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
    let mut s = Session::new_for_test("tsxFindAllReferences1", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
