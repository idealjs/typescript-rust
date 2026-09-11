use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_root_symbol_js_doc_aggregation() {
    let content = r#"
declare const distinct: {
    /** first */
    a: number;
} & {
    /** second */
    a: number;
};

declare const duplicate: {
    /** same */
    a: number;
} & {
    /** same */
    a: number;
} & {
    /** third */
    a: number;
};

declare const mixed: {
    /** first */
    a: number;
} & {
    /** second */
    a: number;
} & {
    /** first */
    a: number;
};

distinct./*distinct*/a
duplicate./*duplicate*/a
mixed./*mixed*/a
"#;
    let mut s = Session::new_for_test("quickInfoRootSymbolJSDocAggregation", content);
    fourslash::verify_quick_info_at(&mut s, "distinct", "(property) a: number", "first\nsecond");
    fourslash::verify_quick_info_at(&mut s, "duplicate", "(property) a: number", "same\nthird");
    fourslash::verify_quick_info_at(&mut s, "mixed", "(property) a: number", "first\nsecond");
}
