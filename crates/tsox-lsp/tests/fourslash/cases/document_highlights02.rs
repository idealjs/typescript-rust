use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: f.MarkTestAsStradaServer()"]
#[test]
fn document_highlights02() {
    let content = r#"// @lib: es5
// @Filename: a.ts
function [|foo|] () {
	return 1;
}
[|foo|]();
// @Filename: b.ts
/// <reference path="a.ts"/>
[|foo|]();"#;
    let mut s = Session::new(content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::go_to_file(&mut s, "a.ts");
    fourslash::go_to_file(&mut s, "b.ts");
    fourslash::unsupported("VerifyBaselineDocumentHighlightsWithOptions"); // f.VerifyBaselineDocumentHighlightsWithOptions(t, nil /*preferences*/, []string{"a.ts", "b.ts"}, ToAn
}
