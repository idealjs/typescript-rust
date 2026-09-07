use tsox_lsp::fourslash::{self, Session};

#[test]
fn incremental_resolve_accessor() {
    let content = r#"class c1 {
    get p1(): string {
        return "30";
    }
    set p1(a: number) {
        a = "30";
    }
}
var val = new c1();
var b = val.p1;
/*1*/b;"#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(&mut s, "1", "var b: string", "");
    fourslash::verify_number_of_errors_in_current_file(&mut s, 1);
}
