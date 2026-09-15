use tsox_lsp::fourslash::{self, Session};


#[test]
fn unreachable_code_after_edit() {
    let content = r#"// @allowUnreachableCode: false
// @lib: es2015
// @Filename: /base/browser/browser.ts
export const isStandalone = true;
// @Filename: /base/browser/dom.ts
export function addDisposableListener() {}
// @Filename: /base/browser/window.ts
export const mainWindow = {} as Window;
// @Filename: /workbench.ts
/*before*/import { isStandalone } from './base/browser/browser';
import { addDisposableListener } from './base/browser/dom';
import { mainWindow } from './base/browser/window';

interface ISecretStorageCrypto {
    seal(data: string): Promise<string>;
    unseal(data: string): Promise<string>;
}

export class TransparentCrypto implements ISecretStorageCrypto {
    async seal(data: string): Promise<string> {
        return data;
    }
    async unseal(data: string): Promise<string> {
        return data;
    }
}"#;
    let mut s = Session::new_for_test("unreachableCodeAfterEdit", content);
    fourslash::verify_number_of_errors_in_current_file(&mut s, 0);
    fourslash::go_to_marker(&mut s, "before");
    fourslash::insert(&mut s, "throw new Error('foo');\n");
    fourslash::verify_number_of_errors_in_current_file(&mut s, 1);
    fourslash::go_to_marker(&mut s, "before");
    fourslash::delete_at_caret(&mut s, 24);
    fourslash::verify_number_of_errors_in_current_file(&mut s, 0);
}
