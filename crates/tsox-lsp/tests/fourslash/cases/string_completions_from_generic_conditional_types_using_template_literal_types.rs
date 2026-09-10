use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn string_completions_from_generic_conditional_types_using_template_literal_types() {
    let content = r#"// @stableTypeOrdering: true
// @strict: true
type keyword = "foo" | "bar" | "baz"

type validateString<s> = s extends keyword
    ? s
    : s extends ` + "`" + `${infer left extends keyword}|${infer right}` + "`" + `
    ? right extends keyword
        ? s
        : ` + "`" + `${left}|${keyword}` + "`" + `
    : keyword

type isUnknown<t> = unknown extends t
    ? [t] extends [{}]
        ? false
        : true
    : false

type validate<def> = def extends string
    ? validateString<def>
    : isUnknown<def> extends true
    ? keyword
    : {
          [k in keyof def]: validate<def[k]>
      }
const parse = <def>(def: validate<def>) => def
const shallowExpression = parse("foo|/*ts*/")
const nestedExpression = parse({ prop: "foo|/*ts2*/" })"#;
    let mut s = Session::new_for_test("stringCompletionsFromGenericConditionalTypesUsingTemplateLiteralTypes", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"ts"}, &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"ts2"}, &fourslash.CompletionsExpectedList{
}
