use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_this_keyword() {
    let content = r#"// @noLib: true
/*1*/this;
function f(/*2*/this) {
    return /*3*/this;
    function g(/*4*/this) { return /*5*/this; }
}
class C {
    static x() {
        /*6*/this;
    }
    static y() {
        () => /*7*/this;
    }
    constructor() {
        /*8*/this;
    }
    method() {
        () => /*9*/this;
    }
}
// These are *not* real uses of the 'this' keyword, they are identifiers.
const x = { /*10*/this: 0 }
x./*11*/this;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11")
}
