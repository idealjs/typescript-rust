use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn quick_info_static_prototype_property_on_class() {
    let content = r#"class c1 {
}
class c2<T> {
}
class c3 {
    constructor() {
    }
}
class c4 {
    constructor(param: string);
    constructor(param: number);
    constructor(param: any) {
    }
}
c1./*1*/prototype;
c2./*2*/prototype;
c3./*3*/prototype;
c4./*4*/prototype;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "(property) c1.prototype: c1", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "2", "(property) c2<T>.prototype: c2<any>", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "3", "(property) c3.prototype: c3", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "4", "(property) c4.prototype: c4", "")
}
