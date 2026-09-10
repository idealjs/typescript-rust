use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineHover"]
#[test]
fn quick_info_js_doc() {
    let content = r#"// @target: esnext
/**
 * A constant
 * @deprecated
 */
var foo = "foo";

/**
 * A function
 * @deprecated
 */
function fn() { }

/**
 * A class
 * @deprecated
 */
class C {
    /**
     * A field
     * @deprecated
     */
    field = "field";

    /**
     * A getter
     * @deprecated
     */
    get getter() {
        return;
    }

    /**
     * A method
     * @deprecated
     */
    m() { }

    get a() {
        this.field/*0*/;
        this.getter/*1*/;
        this.m/*2*/;
        foo/*3*/;
        C/*4*//;
        fn()/*5*/;

        return 1;
    }

    set a(value: number) {
        this.field/*6*/;
        this.getter/*7*/;
        this.m/*8*/;
        foo/*9*/;
        C/*10*/;
        fn/*11*/();
    }
}"#;
    let mut s = Session::new_for_test("quickInfoJsDoc", content);
    fourslash::unsupported("VerifyBaselineHover"); // f.VerifyBaselineHover(t)
}
