use tsox_lsp::fourslash::Session;


#[test]
fn references_for_expression_keywords() {
    let content = r#"class C {
    static x = 1;
}
/*new*/new C();
/*void*/void C;
/*typeof*/typeof C;
/*delete*/delete C.x;
/*async*/async function* f() {
    /*yield*/yield C;
    /*await*/await C;
}
"x" /*in*/in C;
undefined /*instanceof*/instanceof C;
undefined /*as*/as C;"#;
    let _s = Session::new_for_test("referencesForExpressionKeywords", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "new", "void", "typeof", "yield", "await", "in", "instanceof", 
}
