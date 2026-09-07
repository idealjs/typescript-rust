use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "distinct", "(property) a: number", "first\nsecond")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "duplicate", "(property) a: number", "same\nthird")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "mixed", "(property) a: number", "first\nsecond")
}
