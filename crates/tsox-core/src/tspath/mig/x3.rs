use crate::stringutil;

pub(crate) fn get_any_extension_from_path_worker(
    path: &str,
    extensions: &[&str],
    ignore_case: bool,
) -> String {
    for extension in extensions {
        let result = try_get_extension_from_path(path, extension, ignore_case);
        if !result.is_empty() {
            return result;
        }
    }
    String::new()
}

fn try_get_extension_from_path(path: &str, extension: &str, ignore_case: bool) -> String {
    let extension = if extension.starts_with('.') {
        extension.to_string()
    } else {
        format!(".{}", extension)
    };
    if path.len() >= extension.len()
        && path.as_bytes()[path.len() - extension.len()] == b'.'
    {
        let path_extension = &path[path.len() - extension.len()..];
        if stringutil::equate_string_case_insensitive(path_extension, &extension)
            || (!ignore_case && path_extension == extension)
        {
            return path_extension.to_string();
        }
    }
    String::new()
}
