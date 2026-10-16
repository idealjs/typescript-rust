use tsox_lsp::fourslash::Session;


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
    let _s = Session::new_for_test("referencesForUnionProperties", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "one", "base", "x")
}
