use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_inherited_properties5() {
    let content = r#"class C extends D {
    /*0*/prop0: string;
    /*1*/prop1: number;
}

class D extends C {
    /*2*/prop0: string;
}

var d: D;
d./*3*/prop0;
d./*4*/prop1;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "0", "1", "2", "3", "4")
}
