use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_in_template_literal_parts_negatives1() {
    let content = r#"`/*0*/ /*1*/$ /*2*/{ /*3*/$/*4*/{ 10 + 1.1 }/*5*/ 12312/*6*/`

`asdasd$/*7*/{ 2 + 1.1 }/*8*/ 12312 /*9*/{/*10*/"#;
    let mut s = Session::new_for_test("completionListInTemplateLiteralPartsNegatives1", content);
    // TODO: f.VerifyCompletions(t, f.Markers(), nil)
    // TODO: }
}
