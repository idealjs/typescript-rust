use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn string_literal_completions_for_generic_conditional_types_using_template_literal_types() {
    let content = r#"type PathOf<T, K extends string, P extends string = ""> =
  K extends ` + "`" + `${infer U}.${infer V}` + "`" + `
    ? U extends keyof T ? PathOf<T[U], V, ` + "`" + `${P}${U}.` + "`" + `> : ` + "`" + `${P}${keyof T & (string | number)}` + "`" + `
    : K extends keyof T ? ` + "`" + `${P}${K}` + "`" + ` : ` + "`" + `${P}${keyof T & (string | number)}` + "`" + `;

declare function consumer<K extends string>(path: PathOf<{a: string, b: {c: string}}, K>) : number;

consumer('b./*ts*/')"#;
    let mut s = Session::new_for_test("stringLiteralCompletionsForGenericConditionalTypesUsingTemplateLiteralTypes", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"ts"}, &fourslash.CompletionsExpectedList{
}
