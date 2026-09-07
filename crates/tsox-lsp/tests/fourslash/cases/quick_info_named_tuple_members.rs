use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn quick_info_named_tuple_members() {
    let content = r#"export type /*1*/Segment = [length: number, count: number];"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "type Segment = [length: number, count: number]", "")
}
