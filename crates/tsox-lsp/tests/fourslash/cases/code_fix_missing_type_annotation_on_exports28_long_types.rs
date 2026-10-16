use tsox_lsp::fourslash::Session;


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn code_fix_missing_type_annotation_on_exports28_long_types() {
    let content = r#"// @strict: false
// @isolatedDeclarations: true
// @declaration: true
export const sessionLoader = {
    async loadSession() {
        if (Math.random() > 0.5) {
            return {
                PROP_1: {
                    name: false,
                },
                PROPERTY_2: {
                    name: 1,
                },
                PROPERTY_3: {
                    name: 1
                },
                PROPERTY_4: {
                    name: 315,
                },
            };
        }

        return {
            PROP_1: {
                name: false,
            },
            PROPERTY_2: {
                name: undefined,
            },
            PROPERTY_3: {
            },
            PROPERTY_4: {
                name: 576,
            },
        };
    },
};"#;
    let _s = Session::new_for_test("codeFixMissingTypeAnnotationOnExports28_long_types", content);
    // TODO: f.VerifyCodeFixAvailable(t, []string{"Add return type 'Promise<{\n    PROP_1: {\n        name: boole
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
    // TODO: }
}
