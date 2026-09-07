use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_references_umd_module_as_global_const() {
    let content = r#"// @Filename: /node_modules/@types/three/three-core.d.ts
export class Vector3 {
    constructor(x?: number, y?: number, z?: number);
    x: number;
    y: number;
}
// @Filename: /node_modules/@types/three/index.d.ts
export * from "./three-core";
export as namespace /*0*/THREE;
// @Filename: /typings/global.d.ts
import * as _THREE from '/*1*/three';
declare global {
    const /*2*/THREE: typeof _THREE;
}
// @Filename: /src/index.ts
export const a = {};
let v = new /*3*/THREE.Vector2();
// @Filename: /tsconfig.json
{
    "compilerOptions": {
        "esModuleInterop": true,
        "outDir": "./build/js/",
        "noImplicitAny": true,
        "module": "es6",
        "target": "es6",
        "allowJs": true,
        "skipLibCheck": true,
        "lib": ["es2016", "dom"],
        "typeRoots": ["node_modules/@types/"],
        "types": ["three"]
 	},
    "files": ["/src/index.ts", "typings/global.d.ts"]
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "0", "1", "2", "3")
}
