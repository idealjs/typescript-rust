use tsox_lsp::fourslash::{self, Session};


#[test]
fn organize_imports_dts_unused_import_with_augmentation() {
    let content = r#"// @Filename: /styled-patch.d.ts
import * as styledComponents from 'styled-components';

declare module 'styled-components' {
    interface ThemedStyledComponentsModule {
        keyframes(): Keyframes;
    }
}
// @Filename: /node_modules/styled-components/index.d.ts
export interface Keyframes {}
export interface ThemedStyledComponentsModule {}"#;
    let mut s = Session::new_for_test("organizeImports_dtsUnusedImportWithAugmentation", content);
    // TODO: f.VerifyOrganizeImports(
}
