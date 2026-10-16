use tsox_lsp::fourslash::{self, Session};


#[test]
fn quickinfo_for_namespace_merge_with_class_constrained_to_self() {
    let content = r#"declare namespace AMap {
    namespace MassMarks {
        interface Data {
            style?: number;
        }
    }
    class MassMarks<D extends MassMarks.Data = MassMarks.Data> {
        constructor(data: D[] | string);
        clear(): void;
    }
}

interface MassMarksCustomData extends AMap.MassMarks./*1*/Data {
    name: string;
    id: string;
}"#;
    let mut s = Session::new_for_test("quickinfoForNamespaceMergeWithClassConstrainedToSelf", content);
    fourslash::verify_quick_info_at(&mut s, "1", "interface AMap.MassMarks<D extends AMap.MassMarks.Data = AMap.MassMarks.Data>.Data", "");
}
