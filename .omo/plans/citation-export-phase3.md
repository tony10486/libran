# Phase 3 Work Plan: Citation Style Templates

## Architecture Overview

### Module Structure
```
src/citation/text/
├── mod.rs                    # updated: declare locale, helpers, templates modules
├── styles.rs                 # updated: add as_str()/from_str(), flip is_implemented() in Wave 2
├── engine.rs                 # slimmed: pure dispatcher, ~60 LOC prod
├── locale.rs                 # NEW: CSL term localization (4 languages)
├── helpers.rs                # NEW: shared formatting helpers
└── templates/
    ├── mod.rs                # NEW: module declarations + re-exports
    ├── apa.rs                # NEW: extracted from engine.rs (Wave 0)
    ├── acs.rs                # NEW: ACS Guide 2022 (Wave 1)
    ├── ama.rs                # NEW: AMA 11th (Wave 1)
    ├── apsa_asa.rs           # NEW: APSA 2018 + ASA 6th/7th (Wave 1)
    ├── chicago.rs            # NEW: Chicago AD + NB + shortened (Wave 1)
    ├── harvard.rs            # NEW: CTR Harvard + Elsevier Harvard (Wave 1)
    ├── ieee.rs               # NEW: IEEE v11.29.2023 (Wave 1)
    ├── mhra.rs               # NEW: MHRA 4th notes (Wave 1)
    ├── mla.rs                # NEW: MLA 9th in-text (Wave 1)
    ├── nature.rs             # NEW: Nature (Wave 1)
    └── vancouver.rs           # NEW: NLM/Vancouver (Wave 1)

src/export/
├── mod.rs                    # updated: declare preferences module
├── export_dialog_state.rs   # updated: load preferences on new(), tests updated
└── preferences.rs            # NEW: save/load last-used style+format combo

tests/
└── citation_style_tests.rs   # NEW: 45 golden-file tests (15 styles × 3 doc types)
```

## Waves

- **Wave 0** (1 task): Foundation — module structure, locale, helpers, APA extraction
- **Wave 1** (7 parallel): 14 style templates + preference persistence
- **Wave 2** (1 task): Integration — wire dispatcher, flip is_implemented(), update dialog
- **Wave 3** (2 parallel): 45 golden-file tests
- **Wave 4**: Final verification

## Key Tradeoffs (ACCEPTED)
1. Wave 1 commits not independently functional — parallel speedup worth it
2. Chicago shortened notes always produces full notes — single-doc context limitation
3. Superscript → `[1]` bracket notation — TUI limitation
4. MHRA Roman numerals uppercase ASCII — TUI limitation

## 10 Atomic Commits
1. `refactor: extract citation templates module structure with locale and helpers`
2. `feat: implement ACS Guide 2022 and AMA 11th citation styles`
3. `feat: implement Nature, IEEE, and Vancouver citation styles`
4. `feat: implement APSA 2018 and ASA 6th/7th citation styles`
5. `feat: implement Chicago 18th citation styles (author-date, notes+bib, shortened notes)`
6. `feat: implement Cite Them Right Harvard and Elsevier Harvard citation styles`
7. `feat: implement MHRA 4th notes and MLA 9th in-text citation styles`
8. `feat: persist last-used export style and format preferences in app_config`
9. `feat: wire up all 15 citation styles and persist export preferences`
10. `test: add 45 golden-file tests for all 15 citation styles`
