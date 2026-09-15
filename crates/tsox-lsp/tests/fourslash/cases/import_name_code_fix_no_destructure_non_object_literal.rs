use tsox_lsp::fourslash::Session;


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn import_name_code_fix_no_destructure_non_object_literal() {
    let content = r#"// @lib: es5
// @target: es2015
// @strict: true
// @esModuleInterop: true
// @Filename: /array.ts
declare const arr: number[];
export = arr;
// @Filename: /class-instance-member.ts
class C { filter() {} }
export = new C();
// @Filename: /object-literal.ts
declare function filter(): void;
export = { filter };
// @Filename: /jquery.d.ts
interface JQueryStatic {
  filter(): void;
}
declare const $: JQueryStatic;
export = $;
// @Filename: /jquery.js
module.exports = {};
// @Filename: /index.ts
filter/**/"#;
    let _s = Session::new_for_test("importNameCodeFix_noDestructureNonObjectLiteral", content);
    // TODO: f.VerifyImportFixModuleSpecifiers(t, "", []string{"./object-literal", "./jquery"}, nil /*preferences
}
