use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.DeleteAtCaret"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyNumberOfErrorsInCurrentFile"); // f.VerifyNumberOfErrorsInCurrentFile(t, 0)
    fourslash::go_to_marker(&mut s, "before");
    fourslash::insert(&mut s, "throw new Error('foo');\n");
    fourslash::unsupported("VerifyNumberOfErrorsInCurrentFile"); // f.VerifyNumberOfErrorsInCurrentFile(t, 1)
    fourslash::go_to_marker(&mut s, "before");
    fourslash::unsupported("DeleteAtCaret"); // f.DeleteAtCaret(t, 24)
    fourslash::unsupported("VerifyNumberOfErrorsInCurrentFile"); // f.VerifyNumberOfErrorsInCurrentFile(t, 0)
}
