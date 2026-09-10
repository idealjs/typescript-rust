use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: f.MarkTestAsStradaServer()"]
#[test]
fn completions_import_sorting_module_specifiers() {
    let content = r#"// @Filename: tsconfig.json
{ "compilerOptions": { "module": "commonjs", "lib": ["es5"] } }
// @Filename: path.d.ts
declare module "path/posix" {
    export function normalize(p: string): string;
}
declare module "path/win32" {
    export function normalize(p: string): string;
}
declare module "path" {
    export function normalize(p: string): string;
}
// @Filename: main.ts
normalize/**/"#;
    let mut s = Session::new_for_test("completionsImport_sortingModuleSpecifiers", content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
