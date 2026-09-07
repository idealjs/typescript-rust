use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineSelectionRanges"]
#[test]
fn smart_selection_js_doc_tags13() {
    let content = r#"let a;
let b: {
    /** Comment */ /*1*/p0: number
    /** Comment */ /*2*/p1: number
    /** Comment */ /*3*/p2: number
};
let c;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineSelectionRanges"); // f.VerifyBaselineSelectionRanges(t)
}
