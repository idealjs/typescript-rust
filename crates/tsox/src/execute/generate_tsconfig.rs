#![allow(unused_imports)]

use super::*;

pub(crate) fn generate_tsconfig(options: &CompilerOptions) -> String {
    let target = crate::tsoptions::script_target_name(options.target).unwrap_or("esnext");
    let module = crate::tsoptions::module_kind_name(options.module).unwrap_or("nodenext");
    let jsx = crate::tsoptions::jsx_emit_name(options.jsx).unwrap_or("react-jsx");
    let module_detection =
        crate::tsoptions::module_detection_name(options.module_detection).unwrap_or("force");

    format!(
        concat!(
            "{{\n",
            "  // Visit https://aka.ms/tsconfig to read more about this file\n",
            "  \"compilerOptions\": {{\n",
            "    // File Layout\n",
            "    //\"rootDir\": \"./src\",\n",
            "    //\"outDir\": \"./dist\",\n",
            "\n",
            "    // Environment Settings\n",
            "    // See also https://aka.ms/tsconfig/module\n",
            "    \"module\": \"{module}\",\n",
            "    \"target\": \"{target}\",\n",
            "    \"types\": [],\n",
            "    // For nodejs:\n",
            "    // \"lib\": [\"esnext\"],\n",
            "    // \"types\": [\"node\"],\n",
            "    // and npm install -D @types/node\n",
            "\n",
            "    // Other Outputs\n",
            "    \"sourceMap\": true,\n",
            "    \"declaration\": true,\n",
            "    \"declarationMap\": true,\n",
            "\n",
            "    // Stricter Typechecking Options\n",
            "    \"noUncheckedIndexedAccess\": true,\n",
            "    \"exactOptionalPropertyTypes\": true,\n",
            "\n",
            "    // Style Options\n",
            "    //\"noImplicitReturns\": true,\n",
            "    //\"noImplicitOverride\": true,\n",
            "    //\"noUnusedLocals\": true,\n",
            "    //\"noUnusedParameters\": true,\n",
            "    //\"noFallthroughCasesInSwitch\": true,\n",
            "    //\"noPropertyAccessFromIndexSignature\": true,\n",
            "\n",
            "    // Recommended Options\n",
            "    \"strict\": true,\n",
            "    \"jsx\": \"{jsx}\",\n",
            "    \"verbatimModuleSyntax\": true,\n",
            "    \"isolatedModules\": true,\n",
            "    \"noUncheckedSideEffectImports\": true,\n",
            "    \"moduleDetection\": \"{module_detection}\",\n",
            "    \"skipLibCheck\": true\n",
            "  }}\n",
            "}}\n"
        ),
        module = module,
        target = target,
        jsx = jsx,
        module_detection = module_detection
    )
}
