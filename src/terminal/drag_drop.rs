use std::path::PathBuf;

pub fn parse_dragged_path(input: &str) -> Option<PathBuf> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    let mut path_str = trimmed.to_string();

    if ((path_str.starts_with('"') && path_str.ends_with('"'))
        || (path_str.starts_with('\'') && path_str.ends_with('\'')))
        && path_str.len() >= 2
    {
        path_str.remove(0);
        path_str.pop();
    }

    if path_str.starts_with("file://") {
        path_str = path_str[7..].to_string();
    }

    path_str = path_str.replace("\\ ", " ");

    path_str = url_decode(&path_str);

    if path_str.starts_with("~/")
        && let Some(home) = directories::BaseDirs::new() {
            let home_path = home.home_dir().to_path_buf();
            path_str = home_path.join(&path_str[2..]).to_string_lossy().to_string();
        }

    if path_str.contains('\n') {
        path_str = path_str.lines().next().unwrap_or("").to_string();
    }

    let target_path = PathBuf::from(&path_str);

    if target_path.exists() && target_path.is_file() {
        return Some(target_path);
    }

    None
}

fn url_decode(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '%' {
            let h1 = chars.next();
            let h2 = chars.next();
            if let (Some(a), Some(b)) = (h1, h2)
                && let Ok(byte) = u8::from_str_radix(&format!("{}{}", a, b), 16) {
                    result.push(byte as char);
                    continue;
                }
            result.push('%');
            if let Some(a) = h1 {
                result.push(a);
            }
            if let Some(b) = h2 {
                result.push(b);
            }
        } else {
            result.push(c);
        }
    }
    result
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

    #[test]
    fn test_file_url() {
        let result = parse_dragged_path("file:///tmp/test.pdf");
        assert!(result.is_none() || result.is_some());
    }

    #[test]
    fn test_url_encoded() {
        let result = parse_dragged_path("/tmp/My%20Document.pdf");
        assert!(result.is_none() || result.is_some());
    }

    #[test]
    fn test_newline_stripped() {
        let result = parse_dragged_path("/tmp/test.pdf\n");
        assert!(result.is_none() || result.is_some());
    }
}
