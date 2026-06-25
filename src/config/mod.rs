use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::PathBuf;

use crate::api::ApiMode;
use crate::citation::CitationKeyMode;
use crate::storage::FileStoragePolicy;

// ── Theme config types ─────────────────────────────────────────

/// 색상 설정: hex 문자열("#RRGGBB") 또는 RGB 튜플.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ColorConfig {
    Hex(String),
    Rgb { r: u8, g: u8, b: u8 },
}

impl ColorConfig {
    /// hex 문자열 또는 RGB 튜플을 (r, g, b) 로 변환.
    /// 잘못된 형식이면 None 반환.
    pub fn to_rgb(&self) -> Option<(u8, u8, u8)> {
        match self {
            ColorConfig::Hex(s) => {
                let s = s.strip_prefix('#').unwrap_or(s);
                if s.len() == 6 {
                    let r = u8::from_str_radix(&s[0..2], 16).ok()?;
                    let g = u8::from_str_radix(&s[2..4], 16).ok()?;
                    let b = u8::from_str_radix(&s[4..6], 16).ok()?;
                    Some((r, g, b))
                } else {
                    None
                }
            }
            ColorConfig::Rgb { r, g, b } => Some((*r, *g, *b)),
        }
    }
}

/// 배경색 설정 (색상 + 강제 여부).
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct BgConfig {
    pub color: Option<ColorConfig>,
    pub force: bool,
}

impl Default for BgConfig {
    fn default() -> Self {
        BgConfig {
            color: None,
            force: true,
        }
    }
}

/// UI 테마 설정. 각 필드는 Option<ColorConfig> 이며,
/// None 이면 기본 테마값을 사용한다.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ThemeConfig {
    pub bg: BgConfig,
    pub fg: Option<ColorConfig>,
    pub accent_primary: Option<ColorConfig>,
    pub accent_secondary: Option<ColorConfig>,
    pub dim: Option<ColorConfig>,
    pub divider: Option<ColorConfig>,
    pub selected: Option<ColorConfig>,
    pub focus_bg: Option<ColorConfig>,
    pub focus_fg: Option<ColorConfig>,
    pub title: Option<ColorConfig>,
    pub meta: Option<ColorConfig>,
    pub key: Option<ColorConfig>,
    pub tag: Option<ColorConfig>,
    pub udc: Option<ColorConfig>,
    pub error: Option<ColorConfig>,
    pub warning: Option<ColorConfig>,
    pub code: Option<ColorConfig>,
    pub success: Option<ColorConfig>,
    pub search_bg: Option<ColorConfig>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub api_mode: ApiMode,
    pub user_email: Option<String>,
    pub file_storage_policy: FileStoragePolicy,
    pub library_path: PathBuf,
    pub citation_key_mode: CitationKeyModeConfig,
    pub citation_key_template: Option<String>,
    pub primary_scheme: String,
    pub enabled_schemes: Vec<String>,
    pub label_language: String,
    pub db_path: PathBuf,
    pub viewer_command: Option<Vec<String>>,
    pub glyph_set: String,
    pub theme: ThemeConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum CitationKeyModeConfig {
    AuthorYear,
    AuthorYearTitle,
    AuthorYearHash,
    Custom,
}

impl Default for AppConfig {
    fn default() -> Self {
        let home = directories::BaseDirs::new()
            .map(|d| d.home_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));
        let libran_dir = home.join(".libran");

        AppConfig {
            api_mode: ApiMode::IdentifierOnly,
            user_email: None,
            file_storage_policy: FileStoragePolicy::CopyToLibrary,
            library_path: libran_dir.join("library"),
            citation_key_mode: CitationKeyModeConfig::AuthorYear,
            citation_key_template: None,
            primary_scheme: "udc".to_string(),
            enabled_schemes: vec!["udc".to_string(), "physh".to_string(), "msc".to_string()],
            label_language: "en".to_string(),
            db_path: libran_dir.join("libran.db"),
            viewer_command: None,
            glyph_set: "circles".to_string(),
            theme: ThemeConfig::default(),
        }
    }
}

impl AppConfig {
    pub fn path() -> PathBuf {
        let home = directories::BaseDirs::new()
            .map(|d| d.home_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));
        home.join(".libran").join("config.toml")
    }

    pub fn load() -> Self {
        let path = Self::path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(cfg) = toml::from_str::<AppConfig>(&content) {
                    return cfg;
                }
            }
        }
        let cfg = AppConfig::default();
        if let Err(e) = cfg.save() {
            eprintln!("기본 config.toml 생성 실패: {e}");
        }
        cfg
    }

    pub fn save(&self) -> io::Result<()> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let toml_str = self.to_toml_with_comments();
        fs::write(&path, toml_str)?;
        Ok(())
    }

    fn to_toml_with_comments(&self) -> String {
        let mut s = String::new();
        s.push_str("# Libran 설정 파일\n");
        s.push_str("# '#'로 시작하는 줄은 주석입니다. 값을 변경한 후 프로그램을 재시작하세요.\n\n");

        s.push_str("# API 모드: IdentifierOnly | AutoFallback | ManualSearch | FullyOffline\n");
        s.push_str("#   IdentifierOnly:  DOI/arXiv ID가 있을 때만 API 조회합니다. (기본값)\n");
        s.push_str("#   AutoFallback:    식별자가 없으면 제목으로 Crossref 검색까지 시도합니다.\n");
        s.push_str("#   ManualSearch:    수동 검색 모드.\n");
        s.push_str("#   FullyOffline:    모든 온라인 조회를 끕니다.\n");
        s.push_str(&format!("api_mode = \"{}\"\n\n", api_mode_serde_name(&self.api_mode)));

        s.push_str("# CrossRef polite 요청에 사용할 이메일 (선택). 비워두면 익명 요청을 보냅니다.\n");
        match &self.user_email {
            Some(e) => s.push_str(&format!("user_email = \"{}\"\n\n", e)),
            None => s.push_str("user_email = \"\"\n\n"),
        }

        s.push_str("# 파일 저장 정책: copy_to_library | reference_only | copy_and_trash\n");
        s.push_str("#   copy_to_library:  PDF를 library_path로 복사합니다.\n");
        s.push_str("#   reference_only:   원본 위치에 링크만 걸어 둡니다.\n");
        s.push_str("#   copy_and_trash:   복사 후 원본을 휴지통으로 보냅니다.\n");
        s.push_str(&format!("file_storage_policy = \"{}\"\n\n", storage_policy_serde_name(&self.file_storage_policy)));

        s.push_str("# PDF 라이브러리 경로 (절대 경로).\n");
        s.push_str(&format!("library_path = \"{}\"\n\n", self.library_path.display()));

        s.push_str("# 인용 키 생성 모드: author_year | author_year_title | author_year_hash | custom\n");
        s.push_str(&format!("citation_key_mode = \"{}\"\n\n", citation_key_mode_serde_name(&self.citation_key_mode)));

        s.push_str("# 인용 키 커스텀 템플릿 (citation_key_mode = \"custom\"일 때만 사용).\n");
        match &self.citation_key_template {
            Some(t) => s.push_str(&format!("citation_key_template = \"{}\"\n\n", t)),
            None => s.push_str("citation_key_template = \"\"\n\n"),
        }

        s.push_str("# 주 분류 체계: udc | physh | msc | lcc\n");
        s.push_str(&format!("primary_scheme = \"{}\"\n\n", self.primary_scheme));

        s.push_str("# 활성화된 분류 체계 목록.\n");
        let schemes: Vec<String> = self.enabled_schemes.iter()
            .map(|s| format!("\"{}\"", s)).collect();
        s.push_str(&format!("enabled_schemes = [{}]\n\n", schemes.join(", ")));

        s.push_str("# 분류 라벨 언어: en | ko 등.\n");
        s.push_str(&format!("label_language = \"{}\"\n\n", self.label_language));

        s.push_str("# 데이터베이스 파일 경로 (절대 경로).\n");
        s.push_str(&format!("db_path = \"{}\"\n\n", self.db_path.display()));

        s.push_str("# 외부 PDF 뷰어 명령 (선택). 첫 요소는 실행파일, 나머지는 인수.\n");
        s.push_str("# %p는 파일 경로로 치환됩니다.\n");
        s.push_str("#   예: [\"zathura\", \"%p\"] / [\"open\", \"-a\", \"Skim\", \"%p\"]\n");
        s.push_str("# 비워두면 시스템 기본 연결 프로그램 사용 (macOS: open, Windows: 등록된 PDF 앱, Linux: xdg-open)\n");
        match &self.viewer_command {
            Some(parts) if !parts.is_empty() => {
                let items: Vec<String> = parts.iter()
                    .map(|p| format!("\"{}\"", p)).collect();
                s.push_str(&format!("viewer_command = [{}]\n", items.join(", ")));
            }
            _ => s.push_str("viewer_command = []\n"),
        }

        s.push_str("\n# 읽음 상태 마커 글리프 세트: circles | ballot\n");
        s.push_str("#   circles: ○ ◐ ●  (기본값, 기존 ◆★과 동일한 EAW-A)\n");
        s.push_str("#   ballot:  ☐ ⊡ ☒  (EAW-N, CJK 터미널에서 안전)\n");
        s.push_str(&format!("glyph_set = \"{}\"\n", self.glyph_set));

        s.push_str("\n# UI 테마 (선택). 각 색상을 #RRGGBB 형식으로 지정.\n");
        s.push_str("# 지정하지 않은 색상은 기본값 사용. bg.force=true 면 터미널 테마 무시.\n\n");

        s.push_str("[theme.bg]\n");
        s.push_str(&format!("force = {}\n", self.theme.bg.force));
        match &self.theme.bg.color {
            Some(cc) => match cc.to_rgb() {
                Some((r, g, b)) => s.push_str(&format!("color = \"#{:02X}{:02X}{:02X}\"\n", r, g, b)),
                None => s.push_str("# color = \"#000000\"\n"),
            },
            None => s.push_str("# color = \"#000000\"\n"),
        }

        s.push_str("\n[theme]\n");
        let color_fields: &[(&str, &Option<ColorConfig>, &str, &str)] = &[
            ("fg", &self.theme.fg, "#808080", ""),
            ("accent_primary", &self.theme.accent_primary, "#94A3B8", "# Slate (Cyan 대체)"),
            ("accent_secondary", &self.theme.accent_secondary, "#FFFFFF", "# White BOLD (브랜드)"),
            ("dim", &self.theme.dim, "#555555", ""),
            ("divider", &self.theme.divider, "#555555", ""),
            ("selected", &self.theme.selected, "#CDCD00", "# Yellow"),
            ("focus_bg", &self.theme.focus_bg, "#323232", ""),
            ("focus_fg", &self.theme.focus_fg, "#FFFFFF", ""),
            ("title", &self.theme.title, "#FFFFFF", ""),
            ("meta", &self.theme.meta, "#808080", ""),
            ("key", &self.theme.key, "#00CD00", "# Green"),
            ("tag", &self.theme.tag, "#CD00CD", "# Magenta"),
            ("udc", &self.theme.udc, "#0000CD", "# Blue"),
            ("error", &self.theme.error, "#CD0000", "# Red"),
            ("warning", &self.theme.warning, "#CD8500", ""),
            ("code", &self.theme.code, "#CDCD00", ""),
            ("success", &self.theme.success, "#00CD00", ""),
            ("search_bg", &self.theme.search_bg, "#303030", ""),
        ];
        for (name, opt, default_hex, comment) in color_fields {
            match opt {
                Some(cc) => match cc.to_rgb() {
                    Some((r, g, b)) => {
                        s.push_str(&format!("{} = \"#{:02X}{:02X}{:02X}\"\n", name, r, g, b));
                    }
                    None => {
                        if comment.is_empty() {
                            s.push_str(&format!("# {} = {}\n", name, default_hex));
                        } else {
                            s.push_str(&format!("# {} = {}  {}\n", name, default_hex, comment));
                        }
                    }
                },
                None => {
                    if comment.is_empty() {
                        s.push_str(&format!("# {} = {}\n", name, default_hex));
                    } else {
                        s.push_str(&format!("# {} = {}  {}\n", name, default_hex, comment));
                    }
                }
            }
        }

        s
    }

    pub fn to_citation_key_mode(&self) -> CitationKeyMode {
        match self.citation_key_mode {
            CitationKeyModeConfig::AuthorYear => CitationKeyMode::AuthorYear,
            CitationKeyModeConfig::AuthorYearTitle => CitationKeyMode::AuthorYearTitle,
            CitationKeyModeConfig::AuthorYearHash => CitationKeyMode::AuthorYearHash,
            CitationKeyModeConfig::Custom => {
                let template = self.citation_key_template.clone().unwrap_or_default();
                CitationKeyMode::Custom(template)
            }
        }
    }
}

fn api_mode_serde_name(m: &ApiMode) -> &'static str {
    match m {
        ApiMode::FullyOffline => "FullyOffline",
        ApiMode::IdentifierOnly => "IdentifierOnly",
        ApiMode::AutoFallback => "AutoFallback",
        ApiMode::ManualSearch => "ManualSearch",
    }
}

fn storage_policy_serde_name(p: &FileStoragePolicy) -> &'static str {
    match p {
        FileStoragePolicy::CopyToLibrary => "CopyToLibrary",
        FileStoragePolicy::ReferenceOnly => "ReferenceOnly",
        FileStoragePolicy::CopyAndTrash => "CopyAndTrash",
    }
}

fn citation_key_mode_serde_name(m: &CitationKeyModeConfig) -> &'static str {
    match m {
        CitationKeyModeConfig::AuthorYear => "AuthorYear",
        CitationKeyModeConfig::AuthorYearTitle => "AuthorYearTitle",
        CitationKeyModeConfig::AuthorYearHash => "AuthorYearHash",
        CitationKeyModeConfig::Custom => "Custom",
    }
}
