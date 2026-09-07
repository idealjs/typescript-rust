use tsox_lsp::fourslash::{self, Session};

#[test]
fn incremental_resolve_constructor_declaration() {
    let content = r#"class c1 {
    private b: number;
    constructor(a: string) {
        this.b = a;
    }
}
var val = new c1("hello");
/*1*/val;"#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(&mut s, "1", "var val: c1", "");
    fourslash::verify_number_of_errors_in_current_file(&mut s, 1);
}
