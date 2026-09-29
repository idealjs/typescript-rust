use tsox_lsp::fourslash::{self, Session};


#[test]
fn auto_import_reexport_of_cross_package_augmentation() {
    let content = r#"// @Filename: /node_modules/vitest/package.json
{ "name": "vitest", "version": "1.0.0", "types": "index.d.ts" }
// @Filename: /node_modules/vitest/index.d.ts
export { AugmentedInterface, uniqueFunction } from "@vitest/expect";
// @Filename: /node_modules/vitest/augmentation.d.ts
export {};
declare module "@vitest/expect" {
    interface AugmentedInterface {
        bar: string;
    }
		function uniqueFunction(): void;
}
// @Filename: /node_modules/@vitest/expect/package.json
{ "name": "@vitest/expect", "version": "1.0.0", "types": "index.d.ts" }
// @Filename: /node_modules/@vitest/expect/index.d.ts
export interface AugmentedInterface {
    baz: number;
}
// @Filename: /tsconfig.json
{ "compilerOptions": { "module": "commonjs", "strict": true } }
// @Filename: /package.json
{ "name": "test", "dependencies": { "vitest": "*" } }
// @Filename: /index.ts
uniqueFunction/**/"#;
    let mut s = Session::new_for_test("autoImportReexportOfCrossPackageAugmentation", content);
    // TODO: prefs := lsutil.NewDefaultUserPreferences()
    // TODO: prefs.AutoImportEntrypointDirectorySearch = core.TSTrue
    // TODO: f.Configure(t, prefs)
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.BaselineAutoImportsCompletions(t, []string{""})
}
