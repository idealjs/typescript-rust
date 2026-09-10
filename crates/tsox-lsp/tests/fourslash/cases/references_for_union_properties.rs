use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn references_for_union_properties() {
    let content = r#"interface One {
    common: { /*one*/a: number; };
}

interface Base {
    /*base*/a: string;
    b: string;
}

interface HasAOrB extends Base {
    a: string;
    b: string;
}

interface Two {
    common: HasAOrB;
}

var x : One | Two;

x.common./*x*/a;"#;
    let mut s = Session::new_for_test("referencesForUnionProperties", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "one", "base", "x")
}
