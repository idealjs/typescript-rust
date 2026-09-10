use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineHover"]
#[test]
fn quick_info_js_doc_getter_setter() {
    let content = r#"class A {
    /**
     * getter A
     * @returns return A
     */
    get /*1*/x(): string {
        return "";
    }
    /**
     * setter A
     * @param value foo A
     * @todo empty jsdoc
     */
    set /*2*/x(value) { }
}
// override both getter and setter
class B extends A {
    /**
     * getter B
     * @returns return B
     */
    get /*3*/x(): string {
        return "";
    }
    /**
     * setter B
     * @param value foo B
     */
    set /*4*/x(vale) { }
}
// not override
class C extends A { }
// only override setter
class D extends A {
    /**
     * setter D
     * @param value foo D
     */
    set /*5*/x(val: string) { }
}
new A()./*6*/x = "1";
new B()./*7*/x = "1";
new C()./*8*/x = "1";
new D()./*9*/x = "1";"#;
    let mut s = Session::new_for_test("quickInfoJsDocGetterSetter", content);
    fourslash::unsupported("VerifyBaselineHover"); // f.VerifyBaselineHover(t)
}
