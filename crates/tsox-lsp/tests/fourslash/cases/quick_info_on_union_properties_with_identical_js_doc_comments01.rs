use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineHover"]
#[test]
fn quick_info_on_union_properties_with_identical_js_doc_comments01() {
    let content = r#"export type DocumentFilter = {
    /** A language id, like ` + "`" + `typescript` + "`" + `. */
    language: string;
    /** A Uri [scheme](#Uri.scheme), like ` + "`" + `file` + "`" + ` or ` + "`" + `untitled` + "`" + `. */
    scheme?: string;
    /** A glob pattern, like ` + "`" + `*.{ts,js}` + "`" + `. */
    pattern?: string;
} | {
    /** A language id, like ` + "`" + `typescript` + "`" + `. */
    language?: string;
    /** A Uri [scheme](#Uri.scheme), like ` + "`" + `file` + "`" + ` or ` + "`" + `untitled` + "`" + `. */
    scheme: string;
    /** A glob pattern, like ` + "`" + `*.{ts,js}` + "`" + `. */
    pattern?: string;
} | {
    /** A language id, like ` + "`" + `typescript` + "`" + `. */
    language?: string;
    /** A Uri [scheme](#Uri.scheme), like ` + "`" + `file` + "`" + ` or ` + "`" + `untitled` + "`" + `. */
    scheme?: string;
    /** A glob pattern, like ` + "`" + `*.{ts,js}` + "`" + `. */
    pattern: string;
};

declare let x: DocumentFilter;
x./**/language"#;
    let mut s = Session::new_for_test("quickInfoOnUnionPropertiesWithIdenticalJSDocComments01", content);
    fourslash::unsupported("VerifyBaselineHover"); // f.VerifyBaselineHover(t)
}
