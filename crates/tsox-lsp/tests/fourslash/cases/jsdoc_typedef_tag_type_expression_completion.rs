use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "type1", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "typeFooMember", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "NamespaceMember", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "globalValue", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "valueMemberOfSomeType", nil)
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "valueMemberOfFooInstance", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "valueMemberOfFoo", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "propertyName", nil)
}
