use tsox_lsp::fourslash::{self, Session};


#[test]
fn find_all_refs_for_import_call() {
    let content = r#"// @Filename: /app.ts
export function he/**/llo() {};
// @Filename: /re-export.ts
export const services = { app: setup(() => import('./app')) }
function setup<T>(importee: () => Promise<T>): T { return {} as any }
// @Filename: /indirect-use.ts
import("./re-export").then(mod => mod.services.app.hello());
// @Filename: /direct-use.ts
async function main() {
    const mod = await import("./app")
    mod.hello();
}"#;
    let mut s = Session::new_for_test("findAllRefsForImportCall", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "")
}
