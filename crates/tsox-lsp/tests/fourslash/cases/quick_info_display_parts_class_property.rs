use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineHover"]
#[test]
fn quick_info_display_parts_class_property() {
    let content = r#"class c {
    public /*1*/publicProperty: string;
    private /*2*/privateProperty: string;
    protected /*21*/protectedProperty: string;
    static /*3*/staticProperty: string;
    private static /*4*/privateStaticProperty: string;
    protected static /*41*/protectedStaticProperty: string;
    method() {
        this./*5*/publicProperty;
        this./*6*/privateProperty;
        this./*61*/protectedProperty;
        c./*7*/staticProperty;
        c./*8*/privateStaticProperty;
        c./*81*/protectedStaticProperty;
    }
}
var cInstance = new c();
/*9*/cInstance./*10*/publicProperty;
/*11*/c./*12*/staticProperty;"#;
    let mut s = Session::new_for_test("quickInfoDisplayPartsClassProperty", content);
    fourslash::unsupported("VerifyBaselineHover"); // f.VerifyBaselineHover(t)
}
