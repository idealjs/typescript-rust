use tsox_lsp::fourslash::{self, Session};


#[test]
fn jsdoc_typedef_tag_type_expression_completion() {
    let content = r#"// @lib: es5
interface I {
    age: number;
}
 class Foo {
     property1: string;
     constructor(value: number) { this.property1 = "hello"; }
     static method1() {}
     method3(): number { return 3; }
     /**
      * @param {string} foo A value.
      * @returns {number} Another value
      * @mytag
      */
     method4(foo: string) { return 3; }
 }
 namespace Foo.Namespace { export interface SomeType { age2: number } }
 /**
  * @type { /*type1*/Foo./*typeFooMember*/Namespace./*NamespaceMember*/SomeType }
  */
var x;
/*globalValue*/
x./*valueMemberOfSomeType*/
var x1: Foo;
x1./*valueMemberOfFooInstance*/;
Foo./*valueMemberOfFoo*/;
 /**
  * @type { {/*propertyName*/ageX: number} }
  */
var y;"#;
    let mut s = Session::new_for_test("jsdocTypedefTagTypeExpressionCompletion", content);
    // TODO: f.VerifyCompletions(t, "type1", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "typeFooMember", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "NamespaceMember", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "globalValue", &fourslash.CompletionsExpectedList{
    fourslash::verify_completions_empty_at(&mut s, Some("valueMemberOfSomeType"));
    // TODO: f.VerifyCompletions(t, "valueMemberOfFooInstance", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "valueMemberOfFoo", &fourslash.CompletionsExpectedList{
    fourslash::verify_completions_empty_at(&mut s, Some("propertyName"));
}
