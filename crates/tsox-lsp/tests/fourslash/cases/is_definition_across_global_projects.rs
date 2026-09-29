use tsox_lsp::fourslash::Session;


#[test]
fn is_definition_across_global_projects() {
    let content = r#"// @Filename: /home/src/workspaces/project/a/index.ts
/// <reference path="../b/index.ts" />
/// <reference path="../c/index.ts" />
namespace NS {
    export function /*1*/FA() {
        FB();
    }
}

interface /*2*/I {
    /*3*/FA();
}

const ia: I = {
    FA() { },
    FB() { },
    FC() { },
 };
// @Filename: /home/src/workspaces/project/a/tsconfig.json
{
    "extends": "../tsconfig.settings.json",
    "references": [
        { "path": "../b" },
        { "path": "../c" },
    ],
    "files": [
        "index.ts",
    ],
}
// @Filename: /home/src/workspaces/project/b/index.ts
namespace NS {
    export function /*4*/FB() {}
}

interface /*5*/I {
    /*6*/FB();
}

const ib: I = { FB() {} };
// @Filename: /home/src/workspaces/project/b/tsconfig.json
{
    "extends": "../tsconfig.settings.json",
    "files": [
        "index.ts",
    ],
}
// @Filename: /home/src/workspaces/project/c/index.ts
namespace NS {
    export function /*7*/FC() {}
}

interface /*8*/I {
    /*9*/FC();
}

const ic: I = { FC() {} };
// @Filename: /home/src/workspaces/project/c/tsconfig.json
{
    "extends": "../tsconfig.settings.json",
    "files": [
        "index.ts",
    ],
}
// @Filename: /home/src/workspaces/project/tsconfig.json
{
    "compilerOptions": {
        "composite": true,
        "lib": ["es5"],
    },
    "references": [
        { "path": "a" },
    ],
    "files": []
}
// @Filename: /home/src/workspaces/project/tsconfig.settings.json
{
    "compilerOptions": {
        "composite": true,
        "skipLibCheck": true,
        "declarationMap": true,
        "emitDeclarationOnly": true,
        "lib": ["es5"],
    }
}"#;
    let _s = Session::new_for_test("isDefinitionAcrossGlobalProjects", content);
    // TODO: f.MarkTestAsStradaServer()
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4", "5", "6", "7", "8", "9")
}
