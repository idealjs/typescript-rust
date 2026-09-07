use tsox_lsp::fourslash::{self, Session};

#[test]
fn quick_info_named_tuple_members() {
    let content = r#"export type /*1*/Segment = [length: number, count: number];"#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(
        &mut s,
        "1",
        "type Segment = [length: number, count: number]",
        "",
    );
}
