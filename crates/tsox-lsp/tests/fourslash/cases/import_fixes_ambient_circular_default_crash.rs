use tsox_lsp::fourslash::Session;


#[test]
fn import_fixes_ambient_circular_default_crash() {
    let content = r#"// @Filename: /home/src/workspaces/project/tsconfig.json
{
  "compilerOptions": {
    "module": "preserve",
    "lib": ["es5"]
  }
}
// @Filename: /home/src/workspaces/project/types.d.ts
declare module "mymod" {
  import mymod from "mymod";
  export default mymod;
}
// @Filename: /home/src/workspaces/project/index.ts
my/**/"#;
    let _s = Session::new_for_test("importFixes_ambientCircularDefaultCrash", content);
    // TODO: f.MarkTestAsStradaServer()
    // TODO: f.VerifyImportFixModuleSpecifiers(t, "", []string{}, nil /*preferences*/)
}
