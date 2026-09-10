use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: f.VerifyOrganizeImports("]
#[test]
fn organize_imports_with_trace_resolution1() {
    let content = r#"// @Filename: /project/tsconfig.json
{
  "compilerOptions": {
    "traceResolution": true
  }
}
// @Filename: /project/main.ts
import "./dep.js";
"#;
    let mut s = Session::new_for_test("organizeImportsWithTraceResolution1", content);
    fourslash::go_to_file(&mut s, "/project/main.ts");
    // TODO: f.VerifyOrganizeImports(
}
