use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_missing_type_annotation_on_exports45_decorators() {
    let content = r#"// @isolatedDeclarations: true
// @declaration: true
// @Filename: /code.ts
function classDecorator<T extends Function> (value: T, context: ClassDecoratorContext) {}
function methodDecorator<This> (
  target: (...args: number[])=> number,
  context: ClassMethodDecoratorContext<This, (this: This, ...args: number[]) => number>) {}
function getterDecorator(value: Function, context: ClassGetterDecoratorContext) {}
function setterDecorator(value: Function, context: ClassSetterDecoratorContext) {}
function fieldDecorator(value: undefined, context: ClassFieldDecoratorContext) {}
function foo() { return 42;}

@classDecorator
export class A {
  @methodDecorator
  sum(...args: number[]) {
    return args.reduce((a, b) => a + b, 0);
  }
  getSelf() {
    return this;
  }
  @getterDecorator
  get a() {
    return foo();
  }
  @setterDecorator
  set a(value) {}

  @fieldDecorator classProp = foo();
}"#;
    let _s = Session::new_for_test("codeFixMissingTypeAnnotationOnExports45_decorators", content);
    // TODO: f.VerifyCodeFixAll(t, fourslash.VerifyCodeFixAllOptions{
}
