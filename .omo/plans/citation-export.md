# Citation Export with Auto-Clipboard Copy — Work Plan

**Source**: Hyperplan adversarial analysis (degraded: 1/4 member analyses completed, 3 synthesized)
**Plan Agent**: ses_116dda01effeZDQAnQx7B0KiLa
**Created**: 2026-06-21

## Context

Libran is a Rust TUI bibliography manager. The user wants a beautiful export dialog with:
- 15 citation styles (text rendering)
- 16 export formats (data serialization)
- Language selector
- Footnotes/endnotes display mode
- Auto-clipboard copy

**Key architecture insight**: Styles and formats are orthogonal. Styles render citation text (for clipboard). Formats serialize structured data (for file export). Only 2 formats (Bookmarks, Evernote) are style-dependent. The dialog shows all 4 selectors; style affects clipboard text, format affects file export.

**Current state verified**:
- `src/export/mod.rs`: `ExportFormat` enum with 2 variants (Bibtex, CslJson), `export()` dispatch fn
- `src/citation/bibtex.rs`: `export_bibtex()` (46 lines)
- `src/citation/csl_json.rs`: `export_csl_json()` (86 lines)
- `src/app/dispatcher.rs:468`: 'x' sends `ExportRequested(Bibtex)` → `handle_export` writes to `~/export.bib`
- `src/app/dispatcher.rs:268`: `handle_key` dispatches to modal handlers by state flags
- `src/app/dispatcher.rs:1245`: `is_modal_active()` checks all modal flags
- `src/app/state.rs`: `AppState` with 38+ modal/field flags, no export dialog state
- `src/ui/layout.rs:28`: `render()` renders overlays conditionally at lines 72-101
- `src/ui/help.rs`: modal pattern — `centered_rect(55,80)` + `Clear` + no borders + Cyan UNDERLINED headers + ▸ sub-headers
- `src/ui/theme.rs`: Cyan=accent, White=titles, Gray=meta, Yellow=code/selected, Green=success, DarkGray=dim/focus-bg, Red=error, Black=bg
- `src/db/documents.rs`: `Document` struct with 15 fields — NO volume/issue/pages/publisher/city/edition/isbn/issn/url/accessed_date
- `src/db/migrations.rs`: Migration version 8, pattern uses `get_version`/`set_version` in `app_config`
- `Cargo.toml`: No `arboard`, no `serde_yaml`
- `tests/citation.rs`: Integration test pattern using `libran::` paths + `Document` struct + `Cursor` for output

**New deps needed**: `arboard = "3"` (clipboard, Phase 1), `serde_yaml = "0.9"` (CFF, Phase 2)

## Task Dependency Graph

| Task | Depends On | Reason |
|------|------------|--------|
| T1: Add arboard dep | None | Cargo.toml only, no code dependency |
| T2: Extend ExportFormat enum | None | Self-contained enum + methods |
| T3: RIS writer | T2 | Needs ExportFormat enum; research is self-contained |
| T4: CSV writer | T2 | Needs ExportFormat enum |
| T5: MODS writer | T2 | Needs ExportFormat enum; research is self-contained |
| T6: Style renderer + APA 7th | None | Independent of ExportFormat; CitationStyle enum is self-contained |
| T7: ExportDialogState | T2, T6 | Needs ExportFormat + CitationStyle enums |
| T8: Export dialog UI | T6, T7 | Needs style renderer (for preview) + dialog state |
| T9: Wire up dispatcher + clipboard | T1, T3, T4, T5, T6, T8 | Integrates all: deps, writers, renderer, UI, state |
| T10: Update help text | T9 | Help text reflects new 'x' behavior |

## Parallel Execution Graph

```
Wave 1 (Start immediately):
├── T1: Add arboard dep to Cargo.toml (no deps)
└── T2: Extend ExportFormat enum to 16 variants (no deps)

Wave 2 (After Wave 1 completes):
├── T3: Research RIS spec + implement RIS writer (depends: T2)
├── T4: Implement CSV writer (depends: T2)
├── T5: Research MODS spec + implement MODS writer (depends: T2)
├── T6: Research APA 7th CSL + implement style renderer engine (no dep on T2)
└── T7: Create ExportDialogState (depends: T2, T6)

Wave 3 (After Wave 2 completes):
└── T8: Create export dialog UI (depends: T6, T7)

Wave 4 (After Wave 3 completes):
└── T9: Wire up dispatcher + clipboard (depends: T1, T3, T4, T5, T6, T8)

Wave 5 (After Wave 4 completes):
└── T10: Update help text (depends: T9)

Critical Path: T2 → T7 → T8 → T9 → T10
Alt Critical Path: T6 → T7 → T8 → T9 → T10
Estimated Parallel Speedup: ~45% faster than sequential (Wave 2 has 5 parallel tasks)
```

## Tasks

### Task 1: Add arboard clipboard dependency

**Description**: Add `arboard = "3"` to Cargo.toml dependencies. Run `cargo check` to verify it compiles.

**Delegation**: Category: `quick`, Skills: [`programming`]
**Depends On**: None
**Acceptance Criteria**: `cargo check` passes with arboard in dependencies

**File changes**:
- `Cargo.toml`: Add `arboard = "3"` after `urlencoding = "2"` line

**Commit**: `Add arboard clipboard dependency`

---

### Task 2: Extend ExportFormat enum to 16 variants

**Description**: Extend the `ExportFormat` enum from 2 to 16 variants. Add helper methods: `file_extension()`, `format_name()`, `is_style_dependent()`, `is_implemented()`, `all()`. Update `export()` dispatch to handle 5 implemented formats (Bibtex, CslJson, Ris, Csv, Mods) and bail with "not yet implemented" for the remaining 11.

**Delegation**: Category: `quick`, Skills: [`programming`]
**Depends On**: None
**Acceptance Criteria**: `cargo check` passes; `cargo test` passes; all 16 variants compile; `is_implemented()` returns true for Bibtex/CslJson/Ris/Csv/Mods only

**File changes**:
- `src/export/mod.rs`:
  - Change `ExportFormat` derive to `Clone, Copy, Debug, PartialEq, Eq` (add `Copy, Eq`)
  - Add 14 new variants: `Ris, Csv, Mods, BibliontologyRdf, Bookmarks, Cff, CffReferences, Coins, EndnoteXml, ReferBibix, RefworksTagged, EvernoteExport, Tei, WikidataQuickStatements`
  - Add `impl ExportFormat` with 5 methods:
    ```rust
    pub fn file_extension(&self) -> &str
    pub fn format_name(&self) -> &str
    pub fn is_style_dependent(&self) -> bool  // true for Bookmarks, EvernoteExport
    pub fn is_implemented(&self) -> bool      // true for Bibtex, CslJson, Ris, Csv, Mods
    pub fn all() -> &'static [ExportFormat]   // all 16 in order
    ```
  - Update `export()` dispatch:
    ```rust
    ExportFormat::Ris => crate::citation::formats::ris::export_ris(documents, writer),
    ExportFormat::Csv => crate::citation::formats::csv_export::export_csv(documents, writer),
    ExportFormat::Mods => crate::citation::formats::mods::export_mods(documents, writer),
    _ => anyhow::bail!("{} format not yet implemented", format.format_name()),
    ```
  - Add `pub mod export_dialog_state;`

- `src/citation/mod.rs`: Add `pub mod formats;` and `pub mod text;`
- `src/citation/formats/mod.rs`: Create with `pub mod ris; pub mod csv_export; pub mod mods;` (stub modules)
- `src/citation/text/mod.rs`: Create with `pub mod styles; pub mod engine;` (stub modules)
- Stub files: Create empty `src/citation/formats/{ris,csv_export,mods}.rs` and `src/citation/text/{styles,engine}.rs` with placeholder functions returning `Ok(())` or empty structs

**Commit**: `Extend ExportFormat enum to 16 variants with helper methods`

---

### Task 3: Research RIS spec + implement RIS writer with TDD

**Description**: Research the RIS format specification from refman.com. Implement `export_ris()` in `src/citation/formats/ris.rs` following TDD: write 3 test cases first (journal article, book, conference paper), then implement to pass tests.

**Delegation**: Category: `deep`, Skills: [`programming`]
**Depends On**: T2
**Acceptance Criteria**: `cargo test ris` passes; RIS output contains correct tags (TY, TI, AU, PY, JO, DO, ER); 3 test cases pass

**Research sources**:
- Primary: https://refman.com/support/risformat_intro.asp
- Secondary: https://github.com/zotero/translators (RIS.js)
- Key tags: TY (type), TI (title), AU (author, one per line), PY (year), JO (journal), DO (DOI), AB (abstract), KW (keywords), UR (URL), ER (end record)

**File changes**:
- `src/citation/formats/ris.rs`: Implement `export_ris(documents: &[Document], writer: &mut impl Write) -> Result<()>`
  - TY - type mapping: journal → JOUR, conference → CONF, else → GEN
  - TI - title, AU - one author per line (split by ';'), PY - pub_year
  - JO - journal, DO - doi, AB - abstract, KW - keywords, ER - end record

**Test cases**:
1. Journal article → verify TY-JOUR, TI, AU×2, PY, JO, DO, ER
2. Book → verify TY-GEN
3. Conference paper → verify TY-CONF, BT (conference)

**Commit**: `Add RIS format writer with tests`

---

### Task 4: Implement CSV writer with TDD

**Description**: Implement `export_csv()` in `src/citation/formats/csv_export.rs` using the existing `csv` crate (v1). Follow RFC 4180.

**Delegation**: Category: `deep`, Skills: [`programming`]
**Depends On**: T2
**Acceptance Criteria**: `cargo test csv_export` passes; CSV output has header row + data rows; proper quoting; 3 test cases pass

**File changes**:
- `src/citation/formats/csv_export.rs`: Implement `export_csv(documents: &[Document], writer: &mut impl Write) -> Result<()>`
  - Header: id,title,authors,journal,conference,pub_year,doi,arxiv_id,abstract,keywords,citation_key,source,rating
  - Use csv::Writer::from_writer(writer)
  - None fields → empty cells

**Test cases**:
1. Simple journal article → verify header + 1 data row
2. Commas in title → verify proper CSV quoting
3. Missing fields → verify empty cells

**Commit**: `Add CSV format writer with tests`

---

### Task 5: Research MODS spec + implement MODS XML writer with TDD

**Description**: Research the MODS 3.x schema from loc.gov/standards/mods/v3. Implement `export_mods()` using the existing `quick-xml` crate (v0.36).

**Delegation**: Category: `deep`, Skills: [`programming`]
**Depends On**: T2
**Acceptance Criteria**: `cargo test mods` passes; MODS XML is well-formed; contains correct elements; 3 test cases pass

**Research sources**:
- Primary: https://www.loc.gov/standards/mods/v3/mods-3-7.html
- Secondary: https://github.com/zotero/translators (MODS.js)
- Key elements: `<mods version="3.7">`, `<titleInfo><title>`, `<name type="personal"><namePart>`, `<originInfo><dateCreated>`, `<identifier type="doi">`, `<identifier type="arxiv">`, `<abstract>`, `<classification>`

**File changes**:
- `src/citation/formats/mods.rs`: Implement `export_mods(documents: &[Document], writer: &mut impl Write) -> Result<()>`
  - Root: `<modsCollection version="3.7">`
  - Per doc: `<mods>` with titleInfo, name, originInfo, identifier, abstract, classification

**Test cases**:
1. Journal article → verify mods, titleInfo, name elements
2. Multiple authors → verify multiple name elements
3. DOI + arXiv → verify identifier elements

**Commit**: `Add MODS XML format writer with tests`

---

### Task 6: Research APA 7th CSL + implement style renderer engine with TDD

**Description**: Research the APA 7th edition CSL style file. Create the `CitationStyle` enum (15 variants), `CitationLanguage` enum, `DisplayMode` enum, and the style renderer engine. Implement APA 7th template only.

**Delegation**: Category: `deep`, Skills: [`programming`]
**Depends On**: None (independent of ExportFormat)
**Acceptance Criteria**: `cargo test apa` passes; APA 7th reference list + in-text format correct; `CitationStyle::all()` returns 15 variants; `is_implemented()` returns true for Apa7th only; `is_notes_based()` returns true for Chicago notes + MHRA notes styles

**Research sources**:
- Primary: https://github.com/citation-style-language/styles/blob/main/apa.csl
- APA 7th rules:
  - In-text: 1 author → (Smith, 2023); 2 authors → (Smith & Lee, 2023); 3+ authors → (Smith et al., 2023)
  - Reference list: Author, A. A., & Author, B. B. (2023). Title. Journal, Volume(Issue), pages. DOI
  - Author format: "Last, F." (initials), join with ", ", "&" before last

**File changes**:

- `src/citation/text/styles.rs`:
  ```rust
  #[derive(Clone, Copy, Debug, PartialEq, Eq)]
  pub enum CitationStyle {
      AcsGuide2022, Ama11th, Apa7th, Apsa2018, Asa6th7th,
      Chicago18AuthorDate, Chicago18NotesBib, Chicago18ShortenedNotesBib,
      CiteThemRight12thHarvard, ElsevierHarvardWithTitles,
      IeeeV11_29_2023, Mhra4thNotes, Mla9thInText, Nature, NlmVancouverCitingMedicine2nd,
  }

  impl CitationStyle {
      pub fn display_name(&self) -> &str
      pub fn is_implemented(&self) -> bool  // true for Apa7th only in Phase 1
      pub fn is_notes_based(&self) -> bool  // true for Chicago notes + MHRA notes
      pub fn all() -> &'static [CitationStyle]  // 15 variants in user's order
  }

  #[derive(Clone, Copy, Debug, PartialEq, Eq)]
  pub enum CitationLanguage { English, Korean, Japanese, Chinese }

  #[derive(Clone, Copy, Debug, PartialEq, Eq)]
  pub enum DisplayMode { InText, Footnotes, Endnotes }
  ```

- `src/citation/text/engine.rs`:
  ```rust
  pub fn render_citation(doc: &Document, style: CitationStyle, language: CitationLanguage, display_mode: DisplayMode) -> Result<String>
  pub fn render_in_text_citation(doc: &Document, style: CitationStyle, language: CitationLanguage) -> Result<String>
  ```
  - Implement APA 7th only; bail for unimplemented styles
  - Author format: "Smith, J., & Lee, J." (initials, & before last)
  - 3+ authors: all listed up to 20, then et al.
  - Missing fields silently omitted
  - No date → "(n.d.)"

- `src/citation/text/mod.rs`: Re-exports

**Test cases** (6 total):
1. 1 author → `"Smith, J. (2023). Deep learning. Nature. https://doi.org/10.1234/test"`
2. 2 authors → in-text `"(Smith & Lee, 2023)"`
3. 3+ authors → in-text `"(Smith et al., 2023)"`
4. Book (no journal) → no journal line
5. Conference → uses conference field
6. Missing fields → `(n.d.)` for no date

**Commit**: `Add citation style renderer engine with APA 7th template`

---

### Task 7: Create ExportDialogState

**Description**: Create `src/export/export_dialog_state.rs` with the `ExportDialogState` struct and state transition logic.

**Delegation**: Category: `quick`, Skills: [`programming`]
**Depends On**: T2 (ExportFormat enum), T6 (CitationStyle, CitationLanguage, DisplayMode enums)
**Acceptance Criteria**: `cargo test export_dialog_state` passes; Tab cycles through 5 sections; j/k moves cursor; cursor wraps

**File changes**:
- `src/export/export_dialog_state.rs`:
  ```rust
  pub enum DialogSection { Format, Style, Language, DisplayMode, Preview }

  pub struct ExportDialogState {
      pub selected_format: ExportFormat,
      pub selected_style: CitationStyle,
      pub selected_language: CitationLanguage,
      pub display_mode: DisplayMode,
      pub focused_section: DialogSection,
      pub format_cursor: usize,
      pub style_cursor: usize,
      pub language_cursor: usize,
      pub display_mode_cursor: usize,
      pub preview_text: String,
  }

  impl ExportDialogState {
      pub fn new() -> Self
      pub fn tab_next(&mut self)      // Format → Style → Language → DisplayMode → Preview → Format
      pub fn tab_prev(&mut self)
      pub fn cursor_down(&mut self)   // j
      pub fn cursor_up(&mut self)     // k
      pub fn select_current(&mut self)
      pub fn update_preview(&mut self, doc: &Document)
      pub fn is_style_selector_active(&self) -> bool
      pub fn is_display_mode_active(&self) -> bool  // true only for notes-based styles
  }
  ```

- `src/app/state.rs`: Add `show_export_dialog: bool` and `export_dialog_state: ExportDialogState` fields + init in `new()`
- `src/app/dispatcher.rs`: Add `|| state.show_export_dialog` to `is_modal_active()`

**Commit**: `Add ExportDialogState for export dialog`

---

### Task 8: Create export dialog UI

**Description**: Create `src/ui/export_dialog.rs` implementing `render_export_dialog()`. Follow the existing overlay pattern from `src/ui/help.rs`.

**Delegation**: Category: `visual-engineering`, Skills: [`programming`]
**Depends On**: T6 (style renderer for preview), T7 (ExportDialogState)
**Acceptance Criteria**: `cargo check` passes; dialog renders at centered_rect(70, 70); 4 selector sections visible; live preview shows APA 7th citation; unimplemented formats greyed out; focused section highlighted; footer shows keybindings

**File changes**:

- `src/ui/export_dialog.rs`:
  ```rust
  pub fn render_export_dialog(frame: &mut Frame, area: Rect, state: &AppState)
  ```
  - Layout: title | format selector | style selector | language+display (horizontal) | preview | footer
  - centered_rect(70, 70) + Clear + no borders (follow help.rs pattern)
  - Title: "내보내기 / Export" — Cyan + Bold + Underlined
  - Section headers: ▸ 형식 / ▸ 인용 스타일 / ▸ 언어 / ▸ 표시 — Cyan + Underlined
  - Focused section items: Cyan + Bold, ► cursor prefix
  - Unfocused section items: DarkGray
  - Unimplemented items: DarkGray (greyed out)
  - Preview text: Yellow on Black
  - Footer: Enter 복사 | e 내보내기 | Tab 다음 | Esc 취소

- `src/ui/mod.rs`: Add `pub mod export_dialog;`
- `src/ui/layout.rs`: Add render call after line 100: `if state.show_export_dialog { export_dialog::render_export_dialog(frame, area, state); }`
- Add `use super::export_dialog;` to layout.rs imports

**Commit**: `Add export dialog UI with live preview`

---

### Task 9: Wire up dispatcher + clipboard integration

**Description**: Wire up the 'x' key to open the export dialog. Implement `handle_export_dialog_key()` for dialog navigation. Implement clipboard copy with file fallback.

**Delegation**: Category: `deep`, Skills: [`programming`]
**Depends On**: T1, T3, T4, T5, T6, T8
**Acceptance Criteria**: `cargo test` all pass; `cargo build` zero warnings; 'x' opens dialog; Tab cycles sections; j/k navigates; Enter copies to clipboard; 'e' exports to file; Esc closes; clipboard fallback to `~/.libran/clipboard.txt` on failure

**File changes**:

- `src/app/dispatcher.rs`:
  1. Change 'x' handler (line 468): set `state.show_export_dialog = true` + update preview with first selected doc
  2. Add `if state.show_export_dialog { return handle_export_dialog_key(state, key); }` to `handle_key`
  3. Implement `handle_export_dialog_key()`:
     - Tab/BackTab: cycle sections
     - j/Down: cursor down
     - k/Up: cursor up
     - Enter: copy citation to clipboard
     - 'e': export to file
     - Esc: close dialog
  4. Implement `handle_clipboard_copy()`:
     - Render citation in selected style for all selected docs
     - Copy via `arboard::Clipboard::set_text()`
     - Fallback to `~/.libran/clipboard.txt` on failure
     - Status bar: "✓ 클립보드에 복사됨 (N건)" or fallback message
  5. Implement `copy_to_clipboard()`: arboard wrapper with error handling
  6. Update `handle_export()` filename to use `format.file_extension()`

**Commit**: `Wire up export dialog in dispatcher with clipboard support`

---

### Task 10: Update help text

**Description**: Update the 'x' key description in `src/ui/help.rs`.

**Delegation**: Category: `quick`, Skills: [`programming`]
**Depends On**: T9
**Acceptance Criteria**: `cargo check` passes; help text shows updated description

**File changes**:
- `src/ui/help.rs`: Change `help_line("  x", "BibTeX 내보내기")` to `help_line("  x", "내보내기 대화상자 (인용 복사 + 파일 내보내기)")`

**Commit**: `Update help text for export dialog`

---

## Phase 2 Outline (High-Level)

**Goal**: Implement remaining 11 format writers + DB migration for missing Document fields.

1. DB migration (v9): Add columns `volume, issue, page_start, page_end, publisher, city, edition, isbn, issn, url, accessed_date` to documents table. Update `Document` struct, `insert()`, `get_by_id()`, `update()`, `DOCUMENT_COLS`, `doc_from_row!` macro.
2. Add `serde_yaml = "0.9"` to Cargo.toml (for CFF)
3. Implement 11 format writers (each with TDD, 3 test cases):
   - Bibliontology RDF (oxrdf + oxrdfio)
   - Bookmarks (HTML, style-dependent)
   - CFF (YAML, serde_yaml)
   - CFF References (YAML, serde_yaml)
   - COinS (HTML OpenURL spans)
   - Endnote XML (quick-xml)
   - Refer/BibIX (plain text)
   - RefWorks Tagged (plain text)
   - Simple Evernote Export (XML, style-dependent)
   - TEI (quick-xml)
   - Wikidata QuickStatements (TSV)
4. Flip `is_implemented()` for all 11 formats
5. Golden-file tests for all 16 formats
6. Round-trip test: BibTeX → CSL JSON → BibTeX

**Verification gate**: All 16 formats produce spec-compliant output; DB migration backward-compatible; `cargo test` all pass.

---

## Phase 3 Outline (High-Level)

**Goal**: Implement remaining 14 citation style templates + localization + footnotes/endnotes.

1. Research each CSL style file from github.com/citation-style-language/styles (14 files)
2. Implement 14 style templates (may split into `src/citation/text/templates/{apa,acs,ama,...}.rs`)
3. CSL term localization: `et al.` (English) → `등` (Korean) → `等` (Japanese/Chinese)
4. Footnotes/endnotes rendering for notes-based styles (Chicago notes+bib, Chicago shortened notes, MHRA notes)
5. Enable `DisplayMode::is_applicable()` for all notes-based styles
6. Flip `is_implemented()` for all 14 styles
7. Persist last-used style+format combo in `app_config` table
8. Golden-file tests for all 15 styles × 3 document types = 45 test cases

**Verification gate**: All 15 styles produce spec-compliant citations; localization works for 4 languages; footnotes/endnotes correct for 3 notes-based styles; `cargo test` all pass.

---

## Commit Strategy

**Atomic commits (one per task, in order)**:
1. `Add arboard clipboard dependency`
2. `Extend ExportFormat enum to 16 variants with helper methods`
3. `Add RIS format writer with tests`
4. `Add CSV format writer with tests`
5. `Add MODS XML format writer with tests`
6. `Add citation style renderer engine with APA 7th template`
7. `Add ExportDialogState for export dialog`
8. `Add export dialog UI with live preview`
9. `Wire up export dialog in dispatcher with clipboard support`
10. `Update help text for export dialog`

**Rules**:
- Each commit is self-contained: `cargo check` passes after each
- Commits 3-6 can be reordered (independent format/style writers)
- Commits 7-9 must be in order (state → UI → dispatcher)
- Commit 10 is last

**Post-Phase-1 review**: Run `/review-work` after all 10 commits.

---

## Success Criteria

**Phase 1 is complete when ALL of the following pass**:

1. `cargo build` — compiles with zero warnings
2. `cargo test` — all tests pass
3. `cargo test ris` — RIS writer tests pass
4. `cargo test csv_export` — CSV writer tests pass
5. `cargo test mods` — MODS writer tests pass
6. `cargo test apa` — APA 7th renderer tests pass
7. `cargo test export_dialog_state` — Dialog state tests pass
8. Manual: 'x' opens dialog with 4 selectors + live preview
9. Manual: Tab cycles sections, j/k navigates
10. Manual: Enter copies APA 7th citation to clipboard
11. Manual: 'e' exports to file (BibTeX/CSL JSON/RIS/CSV/MODS)
12. Manual: Esc closes dialog
13. Manual: Unimplemented formats/styles greyed out
14. Manual: Clipboard fallback to `~/.libran/clipboard.txt` works
15. Help text updated for 'x' key

## Execution Instructions

1. **Wave 1** (parallel): T1 + T2
2. **Wave 2** (parallel): T3 + T4 + T5 + T6 + T7
3. **Wave 3**: T8
4. **Wave 4**: T9
5. **Wave 5**: T10
6. **Final QA**: `cargo build && cargo test` + `/review-work`
