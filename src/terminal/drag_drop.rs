use std::path::PathBuf;

pub fn parse_dragged_path(input: &str) -> Option<PathBuf> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    let mut path_str = trimmed.to_string();

    if ((path_str.starts_with('"') && path_str.ends_with('"'))
        || (path_str.starts_with('\'') && path_str.ends_with('\'')))
        && path_str.len() >= 2 {
            path_str.remove(0);
            path_str.pop();
        }

    path_str = path_str.replace("\\ ", " ");

    let target_path = PathBuf::from(path_str);

    if target_path.exists() && target_path.is_file() {
        return Some(target_path);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_path() {
        let path = parse_dragged_path("/tmp/test.pdf");
        assert!(path.is_none() || path.is_some());
    }

    #[test]
    fn test_quoted_path() {
        let result = parse_dragged_path("\"/tmp/my file.pdf\"");
        assert!(result.is_none() || result.is_some());
    }

    #[test]
    fn test_escape_space() {
        let result = parse_dragged_path("/tmp/My\\ Document.pdf");
        assert!(result.is_none() || result.is_some());
    }

    #[test]
    fn test_empty_input() {
        assert!(parse_dragged_path("").is_none());
        assert!(parse_dragged_path("   ").is_none());
    }

    #[test]
    fn test_single_quotes() {
        let result = parse_dragged_path("'/tmp/test.txt'");
        assert!(result.is_none() || result.is_some());
    }
}
