use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn references_bloom_filters2() {
    let content = r#"// @Filename: declaration.ts
var container = { /*1*/42: 1 };
// @Filename: expression.ts
function blah() { return (container[42]) === 2;  };
// @Filename: stringIndexer.ts
function blah2() { container["42"] };
// @Filename: redeclaration.ts
container = { "42" : 18 };"#;
    let mut s = Session::new_for_test("referencesBloomFilters2", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1")
}
