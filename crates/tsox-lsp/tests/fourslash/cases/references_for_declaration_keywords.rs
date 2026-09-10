use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn references_for_declaration_keywords() {
    let content = r#"class Base {}
interface Implemented1 {}
/*classDecl1_classKeyword*/class C1 /*classDecl1_extendsKeyword*/extends Base /*classDecl1_implementsKeyword*/implements Implemented1 {
    /*getDecl_getKeyword*/get e() { return 1; }
    /*setDecl_setKeyword*/set e(v) {}
}
/*interfaceDecl1_interfaceKeyword*/interface I1 /*interfaceDecl1_extendsKeyword*/extends Base { }
/*typeDecl_typeKeyword*/type T = { }
/*enumDecl_enumKeyword*/enum E { }
/*namespaceDecl_namespaceKeyword*/namespace N { }
/*moduleDecl_moduleKeyword*/namespace M { }
/*functionDecl_functionKeyword*/function fn() {}
/*varDecl_varKeyword*/var x;
/*letDecl_letKeyword*/let y;
/*constDecl_constKeyword*/const z = 1;
interface Implemented2 {}
interface Implemented3 {}
class C2 /*classDecl2_implementsKeyword*/implements Implemented2, Implemented3 {}
interface I2 /*interfaceDecl2_extendsKeyword*/extends Implemented2, Implemented3 {}"#;
    let mut s = Session::new_for_test("referencesForDeclarationKeywords", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "classDecl1_classKeyword", "classDecl1_extendsKeyword", "classD
}
