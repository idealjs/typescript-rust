use tsox_lsp::fourslash::{self, Session};


#[test]
fn references_for_class_parameter() {
    let content = r#"var p = 2;

class p { }

class foo {
    constructor (/*1*/public /*2*/p: any) {
    }

    public f(p) {
        this./*3*/p = p;
    }

}

var n = new foo(undefined);
n./*4*/p = null;"#;
    let mut s = Session::new_for_test("referencesForClassParameter", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4")
}
