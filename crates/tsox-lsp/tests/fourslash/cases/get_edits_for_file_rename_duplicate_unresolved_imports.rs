use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: // The pathological file: imports the same unresolvable modu"]
#[test]
fn get_edits_for_file_rename_duplicate_unresolved_imports() {
    // TODO: const numFiles = 60   // simulates a large program
    // TODO: const numImports = 60 // simulates the pathological repeated-import file
    // TODO: var content strings.Builder
    // TODO: content.WriteString("// @Filename: /tsconfig.json\n{ \"compilerOptions\": { \"allowJs\": true } }\n"
    // TODO: for i := range numFiles {
    // TODO: // The pathological file: imports the same unresolvable module many times,
    // TODO: // plus one resolvable relative import that should be updated by the rename.
    // TODO: content.WriteString("// @Filename: /pkg/ugly.ts\n")
    // TODO: for range numImports {
    // TODO: content.WriteString("import { v0 } from \"../src/file0\";\n")
    let mut s = Session::new("");
    // TODO: var expected strings.Builder
    // TODO: for range numImports {
    // TODO: expected.WriteString("import { v0 } from \"../src/file0-renamed\";\n")
    fourslash::unsupported("VerifyWillRenameFilesEdits"); // f.VerifyWillRenameFilesEdits(t, "/src/file0.ts", "/src/file0-renamed.ts", map[string]string{
}
