use tsox_lsp::fourslash::{self, Session};


#[test]
fn references_bloom_filters() {
    let content = r#"// @Filename: declaration.ts
var container = { /*1*/searchProp : 1 };
// @Filename: expression.ts
function blah() { return (1 + 2 + container.searchProp()) === 2;  };
// @Filename: stringIndexer.ts
function blah2() { container["searchProp"] };
// @Filename: redeclaration.ts
container = { "searchProp" : 18 };"#;
    let mut s = Session::new_for_test("referencesBloomFilters", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1")
}
