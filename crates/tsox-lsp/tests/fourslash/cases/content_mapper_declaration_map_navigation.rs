use tsox_lsp::fourslash::{self, Session};


#[test]
fn content_mapper_declaration_map_navigation() {
    let content = r#"
// @Filename: /node_modules/component/package.json
{
    "name": "component",
    "version": "1.0.0",
    "types": "component.d.vue.ts"
}

// @Filename: /node_modules/component/component.vue
export interface ComponentProps { emoji: "😀"; label: string; }
export declare const /*source*/component: ComponentProps;

// @Filename: /node_modules/component/component.d.vue.ts
export interface ComponentProps {
    emoji: "😀";
    label: string;
}
export declare const component: ComponentProps;
//# sourceMappingURL=component.d.vue.ts.map

// @Filename: /node_modules/component/component.d.vue.ts.map
{"version":3,"file":"component.d.vue.ts","sourceRoot":"","sources":["component.vue"],"names":[],"mappings":"AAAA,MAAM,WAAW,cAAc;IAAG,KAAK,EAAE,IAAI,CAAC;IAAC,KAAK,EAAE,MAAM,CAAC;CAAE;AAC/D,MAAM,CAAC,OAAO,CAAC,MAAM,SAAS,EAAE,cAAc,CAAC"}

// @Filename: /main.ts
import { component } from "component";
/*use*/component.label;
"#;
    let mut s = Session::new_for_test("contentMapperDeclarationMapNavigation", content);
    // TODO: f.MarkTestAsStradaServer()
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "use")
    // TODO: f.VerifyBaselineFindAllReferences(t, "use")
}
