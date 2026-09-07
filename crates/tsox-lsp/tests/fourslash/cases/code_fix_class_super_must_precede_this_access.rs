use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn code_fix_class_super_must_precede_this_access() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"class Base{
}
class C extends Base{
    private a:number;
    constructor() {[|
        this.a = 12;
        super();
    |]}
    m() { this.a; } // avoid unused 'a'
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyRangeAfterCodeFix"); // f.VerifyRangeAfterCodeFix(t, `
}
