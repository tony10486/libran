pub fn guess_title(page_text: &str) -> Option<String> {
    let lines: Vec<&str> = page_text.lines().filter(|l| !l.trim().is_empty()).collect();
    if lines.is_empty() {
        return None;
    }

    let mut best_line = lines[0].trim().to_string();
    let mut best_len = best_line.len();

    for line in lines.iter().take(10) {
        let trimmed = line.trim();
        let len = trimmed.len();
        if len > best_len && len < 300 && !trimmed.ends_with('.') {
            best_len = len;
            best_line = trimmed.to_string();
        }
    }

    if best_line.is_empty() || best_line.len() < 10 {
        return None;
    }

    Some(best_line)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guess_title_longest_line() {
        let text = "Short\nThis is a much longer title that should be selected as the paper title\nNext line\n";
        let result = guess_title(text);
        assert!(result.is_some());
        assert!(result.unwrap().contains("much longer title"));
    }

    #[test]
    fn test_guess_title_empty() {
        let result = guess_title("");
        assert!(result.is_none());
    }
}
