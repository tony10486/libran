use anyhow::Result;
use std::collections::HashMap;

#[derive(Clone, Debug, Default)]
pub struct ParsedEntry {
    pub entry_type: String,
    pub citation_key: String,
    pub fields: HashMap<String, String>,
}

/// Parse BibTeX content into a list of entries.
/// Supports @type{key, field = {value}, ...} syntax with
/// braced, quoted, and bare values.
pub fn parse_bibtex(content: &str) -> Result<Vec<ParsedEntry>> {
    let mut entries = Vec::new();
    let chars: Vec<char> = content.chars().collect();
    let mut pos = 0;

    while pos < chars.len() {
        skip_whitespace_and_comments(&chars, &mut pos);

        if pos >= chars.len() || chars[pos] != '@' {
            pos += 1;
            continue;
        }

        pos += 1; // skip '@'

        let entry_type = read_identifier(&chars, &mut pos).to_lowercase();
        if entry_type.is_empty() {
            continue;
        }

        skip_spaces(&chars, &mut pos);

        // Skip @comment, @string, @preamble
        if entry_type == "comment" || entry_type == "string" || entry_type == "preamble" {
            skip_braced_group(&chars, &mut pos);
            continue;
        }

        if pos >= chars.len() || chars[pos] != '{' {
            continue;
        }
        pos += 1; // skip '{'

        skip_spaces(&chars, &mut pos);
        let citation_key = read_identifier(&chars, &mut pos);
        skip_spaces(&chars, &mut pos);

        let mut fields = HashMap::new();

        while pos < chars.len() {
            skip_spaces_and_commas(&chars, &mut pos);

            if pos >= chars.len() || chars[pos] == '}' {
                break;
            }

            let field_name = read_identifier(&chars, &mut pos).to_lowercase();
            if field_name.is_empty() {
                pos += 1;
                continue;
            }

            skip_spaces(&chars, &mut pos);

            if pos >= chars.len() || chars[pos] != '=' {
                continue;
            }
            pos += 1; // skip '='

            skip_spaces(&chars, &mut pos);

            let value = read_value(&chars, &mut pos);
            if !value.is_empty() {
                fields.insert(field_name, value);
            }

            skip_spaces_and_commas(&chars, &mut pos);
        }

        if pos < chars.len() && chars[pos] == '}' {
            pos += 1;
        }

        entries.push(ParsedEntry {
            entry_type,
            citation_key,
            fields,
        });
    }

    Ok(entries)
}

fn skip_whitespace_and_comments(chars: &[char], pos: &mut usize) {
    while *pos < chars.len() {
        if chars[*pos] == '%' {
            while *pos < chars.len() && chars[*pos] != '\n' {
                *pos += 1;
            }
        } else if chars[*pos].is_whitespace() {
            *pos += 1;
        } else {
            break;
        }
    }
}

fn skip_spaces(chars: &[char], pos: &mut usize) {
    while *pos < chars.len() && chars[*pos].is_whitespace() {
        *pos += 1;
    }
}

fn skip_spaces_and_commas(chars: &[char], pos: &mut usize) {
    while *pos < chars.len() && (chars[*pos].is_whitespace() || chars[*pos] == ',') {
        *pos += 1;
    }
}

fn read_identifier(chars: &[char], pos: &mut usize) -> String {
    let mut result = String::new();
    while *pos < chars.len() {
        let c = chars[*pos];
        if c.is_alphanumeric()
            || c == '_'
            || c == '-'
            || c == ':'
            || c == '.'
            || c == '+'
            || c == '/'
        {
            result.push(c);
            *pos += 1;
        } else {
            break;
        }
    }
    result
}

fn skip_braced_group(chars: &[char], pos: &mut usize) {
    if *pos >= chars.len() || chars[*pos] != '{' {
        return;
    }
    *pos += 1;
    let mut depth = 1;
    while *pos < chars.len() && depth > 0 {
        if chars[*pos] == '{' {
            depth += 1;
        } else if chars[*pos] == '}' {
            depth -= 1;
        }
        *pos += 1;
    }
}

fn read_value(chars: &[char], pos: &mut usize) -> String {
    if *pos >= chars.len() {
        return String::new();
    }

    // Concatenate values separated by #
    let mut result = String::new();
    loop {
        skip_spaces(chars, pos);
        if *pos >= chars.len() {
            break;
        }

        let part = match chars[*pos] {
            '{' => read_braced_value(chars, pos),
            '"' => read_quoted_value(chars, pos),
            _ => read_bare_value(chars, pos),
        };

        if !part.is_empty() {
            result.push_str(&part);
        }

        skip_spaces(chars, pos);
        if *pos < chars.len() && chars[*pos] == '#' {
            *pos += 1;
            continue;
        }
        break;
    }

    result.trim().to_string()
}

fn read_braced_value(chars: &[char], pos: &mut usize) -> String {
    if *pos >= chars.len() || chars[*pos] != '{' {
        return String::new();
    }
    *pos += 1;
    let mut result = String::new();
    let mut depth = 1;

    while *pos < chars.len() && depth > 0 {
        match chars[*pos] {
            '{' => {
                depth += 1;
                result.push('{');
            }
            '}' => {
                depth -= 1;
                if depth > 0 {
                    result.push('}');
                }
            }
            '\\' => {
                *pos += 1;
                if *pos < chars.len() {
                    result.push(chars[*pos]);
                }
            }
            c => result.push(c),
        }
        *pos += 1;
    }

    result
}

fn read_quoted_value(chars: &[char], pos: &mut usize) -> String {
    if *pos >= chars.len() || chars[*pos] != '"' {
        return String::new();
    }
    *pos += 1;
    let mut result = String::new();
    let mut depth = 0;

    while *pos < chars.len() {
        match chars[*pos] {
            '{' => {
                depth += 1;
                result.push('{');
            }
            '}' => {
                depth -= 1;
                result.push('}');
            }
            '"' if depth == 0 => {
                *pos += 1;
                break;
            }
            '\\' => {
                *pos += 1;
                if *pos < chars.len() {
                    result.push(chars[*pos]);
                }
            }
            c => result.push(c),
        }
        *pos += 1;
    }

    result
}

fn read_bare_value(chars: &[char], pos: &mut usize) -> String {
    let mut result = String::new();
    while *pos < chars.len() {
        let c = chars[*pos];
        if c == ',' || c == '}' || c.is_whitespace() {
            break;
        }
        result.push(c);
        *pos += 1;
    }
    result
}
