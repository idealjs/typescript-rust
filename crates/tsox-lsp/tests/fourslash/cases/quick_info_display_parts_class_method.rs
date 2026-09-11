use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_display_parts_class_method() {
    let content = r#"class c {
    public /*1*/publicMethod() { }
    private /*2*/privateMethod() { }
    protected /*21*/protectedMethod() { }
    static /*3*/staticMethod() { }
    private static /*4*/privateStaticMethod() { }
    protected static /*41*/protectedStaticMethod() { }
    method() {
        this./*5*/publicMethod();
        this./*6*/privateMethod();
        this./*61*/protectedMethod();
        c./*7*/staticMethod();
        c./*8*/privateStaticMethod();
        c./*81*/protectedStaticMethod();
    }
}
var cInstance = new c();
/*9*/cInstance./*10*/publicMethod();
/*11*/c./*12*/staticMethod();"#;
    let mut s = Session::new_for_test("quickInfoDisplayPartsClassMethod", content);
    // TODO: f.VerifyBaselineHover(t)
}
