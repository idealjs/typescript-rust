use crate::fourslash::api::{line_and_character, uri_of};
use crate::fourslash::session::Session;

// ---- B3 quickinfo ----

fn completion_labels_at(s: &mut Session, marker: Option<&str>, prefix: &str) -> Vec<String> {
    let (file_name, position) = match marker {
        // Go "" 指无名 marker（/**/）：解析产出 name=Some("") 的空名 marker，
        // 兼容 name=None 形态
        Some("") => {
            let m = s
                .data
                .markers
                .iter()
                .find(|m| m.name.as_deref().is_none_or(|n| n.is_empty()))
                .cloned()
                .unwrap_or_else(|| panic!("无名 marker 不存在"));
            (m.file_name.clone(), m.position)
        }
        Some(name) => {
            let m = s.marker(name).clone();
            (m.file_name.clone(), m.position)
        }
        None => {
            let file = s.active_file.clone();
            let pos = s
                .cursor
                .expect("无 marker 且无光标（先 go_to_marker/insert）");
            (file, pos)
        }
    };
    // Go GoToMarker：验证后光标停在 marker 位（后续 Insert 等操作依赖）
    if marker.is_some() {
        s.active_file = file_name.clone();
        s.cursor = Some(position);
    }
    let _ = prefix;
    let content = s.file_content(&file_name).to_string();
    let (line, character) = line_and_character(&content, position);
    let service = s.service.as_ref().expect("无 LanguageService");
    let list = service.provide_completion(
        &uri_of(s, &file_name),
        crate::lsp::lsproto_lsp_basic::Position { line, character },
        &crate::ls::types_completion::CompletionContext::default(),
    );
    // Go fourslash 客户端按 SortText 再 Label 排序（ls.CompareCompletionEntries）
    let mut items = list.items;
    items.sort_by(|a, b| compare_completion_entries(a, b));
    items.into_iter().map(|i| i.label).collect()
}

fn compare_completion_entries(
    a: &crate::ls::types_completion::CompletionItem,
    b: &crate::ls::types_completion::CompletionItem,
) -> std::cmp::Ordering {
    fn ci_then_sensitive(a: &str, b: &str) -> std::cmp::Ordering {
        a.to_lowercase()
            .cmp(&b.to_lowercase())
            .then_with(|| a.cmp(b))
    }
    let at = a.sort_text.as_deref().unwrap_or("");
    let bt = b.sort_text.as_deref().unwrap_or("");
    ci_then_sensitive(at, bt).then_with(|| ci_then_sensitive(&a.label, &b.label))
}

/// Go VerifyCompletions(t, <marker>, nil)：期望补全列表为空
pub fn verify_completions_empty_at(s: &mut Session, marker: Option<&str>) {
    let labels = completion_labels_at(s, marker, "");
    assert!(
        labels.is_empty(),
        "期望空补全列表，实际 {:?}",
        labels
    );
}

/// Go Items.Exact：label 序列严格相等（数量+顺序）
pub fn verify_completions_exact_at(s: &mut Session, marker: Option<&str>, expected: &[&str]) {
    let labels = completion_labels_at(s, marker, "exact");
    let expected: Vec<&str> = expected.to_vec();
    assert_eq!(
        labels, expected,
        "补全 Exact 不符（期望 {} 项，实际 {} 项）",
        expected.len(),
        labels.len()
    );
}

/// Go Items.Unsorted：无序集合精确等价（存在性 + 总数）
pub fn verify_completions_unsorted_at(s: &mut Session, marker: Option<&str>, expected: &[&str]) {
    let labels = completion_labels_at(s, marker, "unsorted");
    let mut actual: Vec<String> = labels.clone();
    let mut remaining: Vec<String> = labels;
    for want in expected {
        let idx = remaining.iter().position(|l| l == want);
        let Some(idx) = idx else {
            panic!("Unsorted 缺少 '{}'\n实际: {:?}", want, actual);
        };
        remaining.remove(idx);
    }
    if !remaining.is_empty() {
        panic!(
            "Unsorted 多出未列入项: {:?}\n实际: {:?}",
            remaining, actual
        );
    }
}

/// Go Items.Includes / Items.Excludes（可组合）：includes 每个 label 必须存在；
/// excludes 每个 label 不得存在
pub fn verify_completions_include_exclude_at(
    s: &mut Session,
    marker: Option<&str>,
    includes: &[&str],
    excludes: &[&str],
) {
    let labels = completion_labels_at(s, marker, "includes/excludes");
    for want in includes {
        assert!(
            labels.iter().any(|l| l == want),
            "Includes 缺少 '{}'\n实际: {:?}",
            want,
            labels
        );
    }
    for ban in excludes {
        assert!(
            !labels.iter().any(|l| l == ban),
            "Excludes 不应出现 '{}'\n实际: {:?}",
            ban,
            labels
        );
    }
}
