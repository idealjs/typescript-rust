use tsox_lsp::fourslash::{self, Session};


#[test]
fn chinese_character_display_in_hover() {
    let content = r#"
interface 中文界面 {
    上居中: string;
    下居中: string;
}

class 中文类 {
    获取中文属性(): 中文界面 {
        return {
            上居中: "上居中",
            下居中: "下居中"
        };
    }
}

let /*instanceHover*/实例 = new 中文类();
let 属性对象 = 实例./*methodHover*/获取中文属性();"#;
    let mut s = Session::new_for_test("chineseCharacterDisplayInHover", content);
    fourslash::verify_quick_info_at(&mut s, "instanceHover", "let 实例: 中文类", "");
    fourslash::verify_quick_info_at(&mut s, "methodHover", "(method) 中文类.获取中文属性(): 中文界面", "");
}

#[ignore = "generator: // Verify that the method displays Chinese characters correc"]
#[test]
fn chinese_character_display_in_union_types() {
    let content = r#"
// Test the original issue: Chinese characters in method parameters should display correctly
class TSLine {
    setLengthTextPositionPreset(/*methodParam*/preset: "上居中" | "下居中" | "右居中" | "左居中"): void {}
}

let lines = new TSLine();
lines./*method*/setLengthTextPositionPreset;"#;
    let mut s = Session::new_for_test("chineseCharacterDisplayInUnionTypes", content);
    // TODO: // Verify that the method displays Chinese characters correctly in hover (this was the original prob
    fourslash::verify_quick_info_at(&mut s, "method", r#"(method) TSLine.setLengthTextPositionPreset(preset: "上居中" | "下居中" | "右居中" | "左居中"): void"#, "");
}
