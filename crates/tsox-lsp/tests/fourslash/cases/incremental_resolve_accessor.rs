use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyNumberOfErrorsInCurrentFile"]
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
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "var b: string", "")
    fourslash::unsupported("VerifyNumberOfErrorsInCurrentFile"); // f.VerifyNumberOfErrorsInCurrentFile(t, 1)
}
