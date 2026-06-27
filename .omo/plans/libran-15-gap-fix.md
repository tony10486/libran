# Libran 15-Gap Fix Plan

> **Provenance**: Plan agent (`ses_0f86f7e9effeYJO4mJP3x0tBNq`), synthesized from adversarial 5-member hyperplan team (scrappy/sweaty/brainiac/artful/deepdiver), Rounds 1-2 cross-attack verified.
>
> **Constraint**: No AI, no web app, no word processor integration, no server/HTTP/IPC. Vim-like self-contained tool faithful to its own functionality.

## Context

Libran is a ~25K-line Rust offline-first CUI bibliography manager (ratatui+crossterm TUI, rusqlite WAL+FTS5, tokio async). From the original 17 gaps, **#3 (citation stubs) and #12 (shared libraries) are removed from scope** — all 15 citation styles are real 131-332 line implementations, and cr-sqlite is dead/maintenance-mode. The remaining 15 gaps are sequenced into 16 tasks (gap #24 CJK is split into heuristic + proper fix).

The codebase has 10 versioned migrations (M1-M10), 22 tables, a flat `documents` table (27 columns), and 60 passing tests. `strsim` is already a Cargo dependency. The `filters_json` column exists but is unused. `find_by_fuzzy_title` already exists using `jaro_winkler`.

## Resolved Disputes

| Dispute | Decision | Rationale |
|---------|----------|-----------|
| #1 item_types | `TEXT NOT NULL DEFAULT 'misc' CHECK(...)` column | 8 CSL types fixed/small — normalized table+FK adds join for zero benefit. CHECK addresses integrity. Zotero uses TEXT enum. |
| #7 plugins | **REJECT** — document existing extensibility + config-driven export registry | ClassificationScheme trait + CustomScheme + app_config already exist. Rhai adds ~200KB. Exec-based needs IPC (excluded). |
| #19 notes | SQLite with markdown + multi-note + $EDITOR | Only 4 caller files. File-based breaks 18 call sites, citation_key unstable. SQLite preserves queryability while $EDITOR provides vim-like editing. |

## Migration Pre-Assignment

| Migration | Task | Description |
|-----------|------|-------------|
| M11 | T5 | `ALTER TABLE tags ADD COLUMN color TEXT` |
| M12 | T6 | `ALTER TABLE documents ADD COLUMN queue_position INTEGER` |
| M13 | T10 | Recreate `document_notes` without UNIQUE, add `note_type` column |
| M14 | T9 | `CREATE TABLE documents_body` + `documents_body_fts` FTS5 virtual table |
| M15 | T11 | `ALTER TABLE documents ADD COLUMN item_type TEXT NOT NULL DEFAULT 'misc' CHECK(...)` |
| M16 | T13 | `CREATE TABLE creators` + backfill from `authors` TEXT |
| M17 | T14 | `CREATE TABLE document_attachments` |

**IMPORTANT**: Tasks in the same wave that add migrations (T5, T6, T9, T10 in Wave 1) will both modify `migrations.rs`. The executor must ensure migration blocks are placed in numerical order (M11 < M12 < M13 < M14) in the `run()` function.

## TODOs

### Wave 1 (Start Immediately - No Dependencies)

- [x] **1. Backup/restore via VACUUM INTO (#23)**
  - What: Add `db::backup::backup_to_path()` using `VACUUM INTO`, `:backup`/`:restore` TUI commands, wire to dispatcher
  - Files: `src/db/backup.rs` (new), `src/app/dispatcher.rs`, `src/ui/command_bar.rs` or equivalent
  - Migration: None
  - Depends: None
  - Blocks: None
  - Category: `quick`
  - Skills: `["programming"]`
  - TDD: `test_backup_creates_valid_db`, `test_backup_preserves_data`
  - Acceptance: `:backup /tmp/test.db` creates valid SQLite file with all tables and data. `cargo test` passes. `cargo clippy` clean.
  - Adversarial classes: stale_state (backup file freshness), dirty_worktree (test artifacts)

- [x] **2. Forward citations persistence (#16)**
  - What: Fix `forward_citations_handler.rs:46` — remove `_` from `_citations`, upsert each as document + insert citation_relations edges
  - Files: `src/app/forward_citations_handler.rs`, `src/db/documents.rs`, `src/db/citation_relations.rs`
  - Migration: None
  - Depends: None
  - Blocks: None
  - Category: `quick`
  - Skills: `["programming"]`
  - TDD: `test_forward_citations_persisted`, `test_forward_citations_dedup`
  - Acceptance: Forward citations fetch creates documents + citation_relations edges. No duplicate documents for same DOI. `cargo test` passes. `cargo clippy` clean.
  - Adversarial classes: stale_state (cached citations), malformed input (malformed ForwardCitation)

- [x] **3. Fuzzy duplicate detection (#15)**
  - What: Add `find_duplicates()` using strsim (already in Cargo.toml), weighted title/author/year, threshold 0.75, call during import
  - Files: `src/db/documents.rs` or `src/similarity/` (new module), `src/app/bulk_import_handler.rs` or equivalent
  - Migration: None
  - Depends: None
  - Blocks: None
  - Category: `quick`
  - Skills: `["programming"]`
  - TDD: `test_fuzzy_dup_similar_title`, `test_fuzzy_dup_below_threshold`, `test_fuzzy_dup_different_year_same_title`
  - Acceptance: Duplicate detection catches same paper re-downloaded with different file hash but similar metadata. False positive rate < 5% at threshold 0.75. `cargo test` passes. `cargo clippy` clean.
  - Adversarial classes: malformed input (empty/malformed metadata), flaky tests (threshold edge cases)

- [x] **4. Smart collections / saved search criteria (#20)**
  - What: Implement `filters_json` parsing as SearchCriteria struct, build WHERE clause, add `execute_search()`, update saved_search_handler
  - Files: `src/db/saved_searches.rs`, `src/app/saved_search_handler.rs` or equivalent, `src/ui/` search panel
  - Migration: None (filters_json column already exists)
  - Depends: None
  - Blocks: None
  - Category: `unspecified-low`
  - Skills: `["programming"]`
  - TDD: `test_criteria_filter_by_tag`, `test_criteria_year_range`, `test_criteria_join_any`, `test_criteria_join_all`
  - Acceptance: Saved search with criteria returns correctly filtered documents. `filters_json` is parsed and applied. `cargo test` passes. `cargo clippy` clean.
  - Adversarial classes: malformed input (malformed filters_json), stale_state (saved search referencing deleted tag)

- [x] **5. Color-coded tags + favorites (#25)**
  - What: Migration M11 `ALTER TABLE tags ADD COLUMN color TEXT`, add color CRUD, render colored tags in TUI, favorites = rating=5 filter
  - Files: `src/db/migrations.rs`, `src/db/tags.rs`, `src/ui/` tag rendering, `src/app/dispatcher.rs`
  - Migration: M11
  - Depends: None
  - Blocks: None
  - Category: `quick`
  - Skills: `["programming"]`
  - TDD: `test_tag_color_set_get`, `test_favorite_filter`
  - Acceptance: Tags display with assigned colors in TUI. Favorite filter (rating=5) works. `cargo test` passes. `cargo clippy` clean.
  - Adversarial classes: stale_state (color referencing deleted tag)

- [x] **6. Reading queue / TBR (#26)**
  - What: Migration M12 `ALTER TABLE documents ADD COLUMN queue_position INTEGER`, add queue CRUD, TUI queue view, expose reading_progress keybinding, update help.rs
  - Files: `src/db/migrations.rs`, `src/db/documents.rs`, `src/ui/` queue view, `src/app/dispatcher.rs`, `src/ui/help.rs`
  - Migration: M12
  - Depends: None
  - Blocks: None
  - Category: `unspecified-low`
  - Skills: `["programming"]`
  - TDD: `test_queue_add_remove`, `test_queue_ordered`, `test_reading_progress_update`
  - Acceptance: Reading queue displays in TUI. Documents can be added/removed/reordered. Reading progress is updatable via keybinding. `cargo test` passes. `cargo clippy` clean.
  - Adversarial classes: stale_state (queue referencing deleted document)

- [x] **7. Classification CSV import (#30)**
  - What: Parse CSV (notation,pref_label,broader,alt_labels,notes), build CustomScheme, call register_scheme, add `:import-classification` command
  - Files: `src/classification/csv_import.rs` (new), `src/app/dispatcher.rs`, `README.md`
  - Migration: None (infrastructure exists)
  - Depends: None
  - Blocks: None
  - Category: `quick`
  - Skills: `["programming"]`
  - TDD: `test_csv_import_flat`, `test_csv_import_hierarchical`, `test_csv_import_duplicate_notation`
  - Acceptance: `:import-classification /path/to/scheme.csv` creates a custom scheme with all nodes. Hierarchy is preserved. `cargo test` passes. `cargo clippy` clean.
  - Adversarial classes: malformed input (malformed CSV), prompt injection (CSV content in notes field)

- [x] **8. CJK heuristic patch (#24a)**
  - What: Add `detect_cjk()` Unicode range check, modify `parse_author` and `first_initial` in helpers.rs for CJK names, suppress initials for CJK
  - Files: `src/citation/text/helpers.rs`, `src/citation/text/templates/apa.rs` and other templates
  - Migration: None
  - Depends: None
  - Blocks: T16
  - Category: `unspecified-low`
  - Skills: `["programming"]`
  - TDD: `test_cjk_no_comma_literal`, `test_cjk_with_comma`, `test_western_unchanged`, `test_mixed_authors`
  - Acceptance: CJK names render without empty initials or mangled family/given split. Western names unchanged (no regression — all 60 existing tests pass). `cargo test` passes. `cargo clippy` clean.
  - Adversarial classes: malformed input (empty author string), flaky tests (Unicode edge cases)

- [x] **9. Full-text body indexing (#13)**
  - What: Migration M14 `CREATE TABLE documents_body + documents_body_fts`, store PDF body text during import, add body FTS search with toggle
  - Files: `src/db/migrations.rs`, `src/db/documents_body.rs` (new), `src/pdf/text.rs`, `src/app/bulk_import_handler.rs`, `src/db/search.rs`
  - Migration: M14
  - Depends: None
  - Blocks: None
  - Category: `unspecified-high`
  - Skills: `["programming"]`
  - TDD: `test_body_stored`, `test_body_fts_search`, `test_body_fts_toggle`
  - Acceptance: PDF body text is stored and searchable. FTS toggle works. Storage ~95kB/entry. `cargo test` passes. `cargo clippy` clean.
  - Adversarial classes: stale_state (body text not updated when PDF replaced), hung commands (large PDF extraction)

- [x] **10. Notes markdown + multi-note + $EDITOR (#19)**
  - What: Migration M13 recreate document_notes without UNIQUE + add note_type, change notes interface (list/create/update/delete_by_id), update state.rs + dispatcher.rs callers, add `:note` $EDITOR command
  - Files: `src/db/migrations.rs`, `src/db/notes.rs`, `src/app/state.rs`, `src/app/dispatcher.rs`, `src/ui/` note panel
  - Migration: M13
  - Depends: None
  - Blocks: None
  - Category: `unspecified-high`
  - Skills: `["programming"]`
  - TDD: `test_multi_note`, `test_note_crud`, `test_note_list_ordered`
  - Acceptance: Multiple notes per document supported. `$EDITOR` opens for note editing. Existing note data preserved after migration. `cargo test` passes. `cargo clippy` clean.
  - Adversarial classes: stale_state (note referencing deleted document), dirty_worktree (temp files from $EDITOR)

### Wave 2 (After Wave 1 Completes)

- [x] **11. Item types — TEXT column with CHECK (#1)**
  - What: Migration M15 `ALTER TABLE documents ADD COLUMN item_type TEXT NOT NULL DEFAULT 'misc' CHECK(...)`, add to Document struct, backfill from metadata, update csl_json.rs/bibtex.rs/ris.rs to use item_type, add to edit mode
  - Files: `src/db/migrations.rs`, `src/db/schema.rs`, `src/db/documents.rs`, `src/citation/csl_json.rs`, `src/citation/bibtex.rs`, `src/citation/formats/ris.rs`, `src/ui/` edit mode
  - Migration: M15
  - Depends: Wave 1 complete
  - Blocks: T13, T14, T15
  - Category: `unspecified-high`
  - Skills: `["programming"]`
  - TDD: `test_item_type_default`, `test_item_type_inferred`, `test_csl_json_uses_item_type`, `test_bibtex_uses_item_type`, `test_item_type_user_override`
  - Acceptance: `item_type` column exists with CHECK constraint. Export formats use `item_type` for type-aware output. Backfill populates existing docs. User can edit item_type. `cargo test` passes (including updated golden file tests). `cargo clippy` clean.
  - Adversarial classes: stale_state (golden file tests need update), malformed input (invalid item_type value)

- [x] **12. Export registry + extensibility docs (#7)**
  - What: Add config-driven custom export format registry, document ClassificationScheme trait + app_config + custom formats in README
  - Files: `src/export/mod.rs`, `src/app/config.rs` or equivalent, `README.md`
  - Migration: None
  - Depends: Wave 1 complete
  - Blocks: None
  - Category: `quick`
  - Skills: `["programming", "writing"]`
  - TDD: `test_custom_format_registered`, `test_custom_format_generates`
  - Acceptance: Custom export format can be registered via config and dispatched. README documents extension points. `cargo test` passes. `cargo clippy` clean.
  - Adversarial classes: malformed input (malformed config template), stale_state (config referencing deleted format)

### Wave 3 (After Wave 2 Completes)

- [x] **13. Structured creators with roles (#17) — HIGHEST RISK**
  - What: Migration M16 `CREATE TABLE creators`, create db/creators.rs CRUD, backfill from authors TEXT, CJK detection with locale/literal fields, dual-write (authors TEXT + creators rows), update 15 files calling split_authors
  - Files: `src/db/migrations.rs`, `src/db/creators.rs` (new), `src/db/documents.rs`, all 15 files calling `split_authors` (citation templates, export formats)
  - Migration: M16
  - Depends: T11
  - Blocks: T15, T16
  - Category: `deep`
  - Skills: `["programming"]`
  - TDD: `test_creator_insert_with_role`, `test_creator_order`, `test_creator_cjk_locale`, `test_dual_write`, `test_backfill`, `test_creators_to_authors_string`
  - Acceptance: `creators` table exists with all fields. Backfill populates from existing `authors` TEXT. Dual-write works. All 15 files that call `split_authors` can use `creators` table. CJK names get `locale` + `literal` fields. All 60 existing tests pass (backward compat). `cargo test` passes. `cargo clippy` clean.
  - Risk mitigation: Dual-write ensures backward compatibility. CJK backfill is lossy — ambiguous names go to `literal` field. 15-file blast radius — update one file at a time, run tests after each.
  - Adversarial classes: stale_state (creators referencing deleted document), malformed input (malformed authors TEXT), dirty_worktree (migration rollback)

- [x] **14. Multi-attachment / non-PDF (#18)**
  - What: Migration M17 `CREATE TABLE document_attachments`, create db/attachments.rs CRUD, modify storage for multi-file, keep documents.file_path as primary (backward compat), add TUI attachment management
  - Files: `src/db/migrations.rs`, `src/db/attachments.rs` (new), `src/storage/library.rs`, `src/ui/` attachment panel
  - Migration: M17
  - Depends: T11
  - Blocks: None
  - Category: `unspecified-high`
  - Skills: `["programming"]`
  - TDD: `test_multiple_attachments`, `test_non_pdf_attachment`, `test_attachment_hash`, `test_primary_attachment_backward_compat`
  - Acceptance: Multiple attachments per document supported. Non-PDF files can be attached. `documents.file_path` still works as primary attachment. `cargo test` passes. `cargo clippy` clean.
  - Adversarial classes: stale_state (attachment referencing deleted document), malformed input (malformed file path)

### Wave 4 (After Wave 3 Completes)

- [x] **15. Full export with user data (#2)**
  - What: Extend all export modules to include notes, tags, classifications, projects, reading_status, custom_fields; add full library JSON export mode
  - Files: `src/export/mod.rs`, `src/citation/csl_json.rs`, `src/citation/bibtex.rs`, `src/citation/formats/ris.rs`, `src/export/csv.rs` or equivalent, all other export format files
  - Migration: None
  - Depends: T11, T13
  - Blocks: None
  - Category: `unspecified-high`
  - Skills: `["programming"]`
  - TDD: `test_csl_json_includes_tags`, `test_csv_includes_notes`, `test_full_library_export`, `test_export_includes_classifications`
  - Acceptance: All export formats include user-created data (notes, tags, classifications, projects, reading_status). Full library export captures everything. `cargo test` passes (update golden file tests). `cargo clippy` clean.
  - Adversarial classes: stale_state (export referencing deleted related data), malformed input (malformed user data)

- [x] **16. CJK proper fix with per-creator locale (#24b)**
  - What: Use creators.locale field for proper CJK rendering, replace T8 heuristic with per-creator detection, update helpers.rs and 15 citation templates to pass locale
  - Files: `src/citation/text/helpers.rs`, all 15 citation template files in `src/citation/text/templates/`
  - Migration: None
  - Depends: T13
  - Blocks: None
  - Category: `unspecified-low`
  - Skills: `["programming"]`
  - TDD: `test_cjk_creator_korean`, `test_cjk_creator_literal`, `test_mixed_cjk_western`, `test_cjk_no_regression`
  - Acceptance: CJK creators render correctly per their `locale` field. Western creators unchanged. All 60 golden file tests pass (update if expected output changes for CJK test cases). `cargo test` passes. `cargo clippy` clean.
  - Adversarial classes: flaky tests (Unicode edge cases), stale_state (golden file updates)

## Final Verification Wave

- [ ] **F1. All migrations run cleanly on fresh DB**
  - Verify: Create fresh DB, run all M11-M17 migrations, verify no errors
  - Command: `cargo test -- --include-ignored migration_test` or equivalent

- [ ] **F2. All migrations upgrade existing DB without data loss**
  - Verify: Use existing test DB, run M11-M17, verify all data preserved
  - Command: `cargo test` with migration upgrade tests

- [ ] **F3. Full test suite passes**
  - Verify: `cargo test` — all tests pass (estimated 60 + ~50 new = ~110 tests)
  - Command: `cargo test`

- [ ] **F4. Lint and format clean**
  - Verify: `cargo clippy` zero warnings, `cargo fmt --check` clean
  - Commands: `cargo clippy`, `cargo fmt --check`

- [ ] **F5. Manual TUI smoke test**
  - Verify: Start TUI, verify backup, forward citations, fuzzy dedup, smart collections, color tags, reading queue, classification import, CJK rendering, body search, multi-note, item types, multi-attachment, full export all work
  - Manual: `cargo run` and exercise each feature

## Risk Register

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| T13 (creators) breaks 15 call sites | Medium | High | Dual-write preserves `authors` TEXT; update one file at a time with tests |
| T13 CJK backfill is lossy | High | Medium | Ambiguous names → `literal` field; user manually fixes |
| Wave 1 migration conflicts in `migrations.rs` | Medium | Low | Pre-assigned M11-M14; executor orders blocks numerically |
| T11 changes break golden file tests | Medium | Medium | Update golden files to reflect correct type-aware output |
| T9 body indexing increases DB size | Low | Low | ~95kB/entry; documented; user can skip body indexing |
| T10 table recreate loses data | Low | High | Migration copies all data before dropping old table; tested |

## Commit Strategy

Atomic commits, one per task. Each commit includes implementation code, tests, and any schema/migration changes.

Pre-commit verification (for every task):
1. `cargo test` — all tests pass (existing + new)
2. `cargo clippy` — no warnings
3. `cargo fmt --check` — formatting clean
