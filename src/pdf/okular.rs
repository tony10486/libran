use std::path::{Path, PathBuf};
use directories::BaseDirs;
use rusqlite::Connection;

/// Okular의 docdata 폴더 경로들을 OS별로 찾아서 반환합니다.
pub fn get_okular_docdata_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    let base_dirs = match BaseDirs::new() {
        Some(d) => d,
        None => return dirs,
    };
    let home = base_dirs.home_dir().to_path_buf();

    #[cfg(target_os = "macos")]
    {
        dirs.push(home.join("Library/Application Support/okular/docdata"));
        dirs.push(home.join("Library/Application Support/KDE/okular/docdata"));
    }

    #[cfg(target_os = "windows")]
    {
        if let Some(appdata) = directories::UserDirs::new().and_then(|u| Some(u.home_dir().to_path_buf())) {
            dirs.push(appdata.join("AppData/Local/okular/docdata"));
            dirs.push(appdata.join("AppData/Roaming/okular/docdata"));
        }
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))] // Linux
    {
        dirs.push(home.join(".local/share/okular/docdata"));
    }

    dirs
}

/// PDF 파일 경로에 대응하는 Okular docdata XML 파일을 탐색하여 마지막 읽은 페이지를 동기화합니다.
/// Okular의 XML 명명 규칙: `<파일크기(bytes)>.<파일명>.xml`
pub fn sync_okular_data(conn: &Connection, doc_id: i64) -> Result<String, String> {
    // 1. DB에서 문헌 정보 조회 (파일 경로)
    let file_path = crate::db::documents::get_by_id(conn, doc_id)
        .map_err(|e| e.to_string())?
        .and_then(|d| d.file_path)
        .ok_or_else(|| "문헌의 파일 경로가 존재하지 않습니다.".to_string())?;

    let path = Path::new(&file_path);
    if !path.exists() {
        return Err(format!("파일이 존재하지 않습니다: {}", file_path));
    }

    // 2. 파일 크기 및 파일명 추출
    let file_size = std::fs::metadata(path)
        .map(|m| m.len())
        .map_err(|e| format!("파일 메타데이터 조회 실패: {}", e))?;
    let file_name = path.file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| "파일명을 추출할 수 없습니다.".to_string())?;

    // 매칭할 XML 파일 이름 (예: "1048576.my_doc.pdf.xml")
    let target_xml_name = format!("{}.{}.xml", file_size, file_name);
    
    // 3. docdata 폴더 순회 탐색
    let docdata_dirs = get_okular_docdata_dirs();
    let mut found_xml_path: Option<PathBuf> = None;

    for dir in docdata_dirs {
        if dir.exists() {
            let candidate = dir.join(&target_xml_name);
            if candidate.exists() {
                found_xml_path = Some(candidate);
                break;
            }
        }
    }

    let xml_path = match found_xml_path {
        Some(p) => p,
        None => return Ok("Okular에서 읽기 진척도 데이터를 찾지 못했습니다.".to_string()),
    };

    // 4. XML 파일 내용 로드 및 단순 파싱 (<current viewport="X;..."/>)
    let content = std::fs::read_to_string(&xml_path)
        .map_err(|e| format!("XML 파일 로드 실패: {}", e))?;

    // <current viewport="X;" 또는 <oldPage viewport="X;" 파싱
    let page_number = parse_page_number_from_xml(&content)
        .unwrap_or(0); // 파싱 실패 시 예외 에러를 반환하지 않고 안전하게 0페이지(1페이지)로 폴백합니다.

    let mut progress_pct = 0;
    let mut total_pages = 0;

    if let Ok(doc) = lopdf::Document::load(&file_path) {
        total_pages = doc.get_pages().len();
        if total_pages > 0 {
            progress_pct = (((page_number + 1) as f64 / total_pages as f64) * 100.0).round().min(100.0) as i64;
        }
    }

    let status = if progress_pct >= 100 { "read" } else { "reading" };

    // 5. DB에 읽기 진척도 및 상태 저장
    conn.execute(
        "UPDATE documents SET reading_progress = ?1, reading_status = ?2, updated_at = datetime('now') WHERE id = ?3",
        rusqlite::params![progress_pct, status, doc_id],
    ).map_err(|e| format!("DB 진척도 업데이트 실패: {}", e))?;

    if total_pages > 0 {
        Ok(format!("Okular 읽기 진척도 동기화 완료: {}% ({} / {} 페이지)", progress_pct, page_number + 1, total_pages))
    } else {
        Ok(format!("Okular 읽기 진척도 동기화 완료: {} 페이지", page_number + 1))
    }
}

/// XML 문자열에서 `current viewport="X;` 또는 `oldPage viewport="X;` 에서 X 값을 추출합니다 (0-indexed).
fn parse_page_number_from_xml(content: &str) -> Option<i64> {
    let key = "current viewport=";
    if let Some(idx) = content.find(key) {
        let remain = &content[idx + key.len()..];
        let quote = remain.chars().next()?;
        if quote == '"' || quote == '\'' {
            let rest = &remain[1..];
            if let Some(end_idx) = rest.find(';') {
                let num_str = &rest[..end_idx];
                if let Ok(num) = num_str.parse::<i64>() {
                    return Some(num);
                }
            }
        }
    }

    let key_old = "oldPage viewport=";
    if let Some(idx) = content.find(key_old) {
        let remain = &content[idx + key_old.len()..];
        let quote = remain.chars().next()?;
        if quote == '"' || quote == '\'' {
            let rest = &remain[1..];
            if let Some(end_idx) = rest.find(';') {
                let num_str = &rest[..end_idx];
                if let Ok(num) = num_str.parse::<i64>() {
                    return Some(num);
                }
            }
        }
    }

    None
}

/// 시스템 PATH 및 표준 설치 경로에서 Okular 실행 파일을 탐색합니다.
pub fn find_okular_executable() -> Option<String> {
    // 1. 시스템 PATH 환경변수 내에서 검사
    if std::process::Command::new("okular")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok()
    {
        return Some("okular".to_string());
    }

    // 2. OS별 기본 설치 위치 하드코딩 탐색
    #[cfg(target_os = "macos")]
    {
        let paths = vec![
            "/Applications/Okular.app/Contents/MacOS/okular",
            "/Applications/okular.app/Contents/MacOS/okular",
            "~/Applications/Okular.app/Contents/MacOS/okular",
            "/opt/homebrew/bin/okular",
            "/usr/local/bin/okular",
        ];
        for p in paths {
            let expanded = if p.starts_with('~') {
                if let Some(home) = BaseDirs::new().map(|d| d.home_dir().to_path_buf()) {
                    home.join(&p[2..])
                } else {
                    PathBuf::from(p)
                }
            } else {
                PathBuf::from(p)
            };
            if expanded.exists() {
                return Some(expanded.to_string_lossy().to_string());
            }
        }

        // Caskroom 하위 버전의 Okular.app 내 실행파일 동적으로 찾기
        if let Ok(entries) = std::fs::read_dir("/opt/homebrew/Caskroom/okular") {
            for entry in entries.flatten() {
                let p = entry.path().join("Okular.app").join("Contents").join("MacOS").join("okular");
                if p.exists() {
                    return Some(p.to_string_lossy().to_string());
                }
                let p_lower = entry.path().join("okular.app").join("Contents").join("MacOS").join("okular");
                if p_lower.exists() {
                    return Some(p_lower.to_string_lossy().to_string());
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        let paths = vec![
            "C:\\Program Files\\Okular\\bin\\okular.exe",
            "C:\\Program Files (x86)\\Okular\\bin\\okular.exe",
        ];
        for p in paths {
            let path = Path::new(p);
            if path.exists() {
                return Some(path.to_string_lossy().to_string());
            }
        }
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))] // Linux
    {
        let paths = vec![
            "/usr/bin/okular",
            "/usr/local/bin/okular",
            "/snap/bin/okular",
        ];
        for p in paths {
            let path = Path::new(p);
            if path.exists() {
                return Some(path.to_string_lossy().to_string());
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_page_number_from_xml() {
        let xml1 = r#"<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE documentInfo>
<documentInfo url="/path/to/doc.pdf">
 <generalInfo>
  <history>
   <current viewport="13;C2:0.5:0.353297:1"/>
  </history>
 </generalInfo>
</documentInfo>"#;
        assert_eq!(parse_page_number_from_xml(xml1), Some(13));

        let xml2 = r#" <oldPage viewport='45;C2:0.5:1'/> "#;
        assert_eq!(parse_page_number_from_xml(xml2), Some(45));

        let xml3 = r#" <current viewport="invalid;"/> "#;
        assert_eq!(parse_page_number_from_xml(xml3), None);
    }
}