use tsox_lsp::fourslash::{self, Session};

#[test]
fn hover_call_signature_documentation() {
    let content = r#"
type X = {
    /** Description of invoking. */
    (): string

    /** Description of constructor. */
    new (): number
}

declare const x: X

/*1*/x()
new /*2*/x()
"#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(
        &mut s,
        "1",
        "const x: () => string",
        "Description of invoking.",
    );
    fourslash::verify_quick_info_at(
        &mut s,
        "2",
        "const x: new () => number",
        "Description of constructor.",
    );
}
