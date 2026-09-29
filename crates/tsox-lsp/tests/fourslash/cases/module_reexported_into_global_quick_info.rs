use tsox_lsp::fourslash::{self, Session};


#[test]
fn module_reexported_into_global_quick_info() {
    let content = r#"// @Filename: /node_modules/@types/three/index.d.ts
export class Vector3 {}
export as namespace THREE;
// @Filename: /global.d.ts
import * as _THREE from 'three';

declare global {
  const THREE: typeof _THREE;
}
// @Filename: /index.ts
let v = new /*1*/THREE.Vector3();"#;
    let mut s = Session::new_for_test("moduleReexportedIntoGlobalQuickInfo", content);
    fourslash::verify_quick_info_at(&mut s, "1", "const THREE: typeof import(\"three\")", "");
}
