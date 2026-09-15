use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_missing_type_annotation_on_exports46_decorators_experimental() {
    let content = r#"// @isolatedDeclarations: true
// @declaration: true
// @experimentalDecorators: true
// @Filename: /code.ts
function classDecorator<T extends Function>() { return (target: T) => target; }
function methodDecorator() { return (target: any, key: string, descriptor: PropertyDescriptor) => descriptor;}
function parameterDecorator() { return (target: any, key: string, idx: number) => {};}
function getterDecorator() { return (target: any, key: string) => {}; }
function setterDecorator() { return (target: any, key: string) => {}; }
function fieldDecorator()  { return (target: any, key: string) => {}; }
function foo() { return 42; }

@classDecorator()
export class A {
  @methodDecorator()
  sum(...args: number[]) {
    return args.reduce((a, b) => a + b, 0);
  }
  getSelf() {
    return this;
  }
  passParameter(@parameterDecorator() param = foo()) {}
  @getterDecorator()
  get a() {
    return foo();
  }
  @setterDecorator()
  set a(value) {}
  @fieldDecorator() classProp = foo();
}"#;
    let _s = Session::new_for_test("codeFixMissingTypeAnnotationOnExports46_decorators_experimental", content);
    // TODO: f.VerifyCodeFixAll(t, fourslash.VerifyCodeFixAllOptions{
}
