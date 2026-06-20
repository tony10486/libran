# Korean Substring Search — Work Plan

## Problem

FTS5 trigram tokenizer creates 3-char tokens only → 2-char Korean queries like "미분" can't match.
`escape_fts_query()` uses byte-length (`term.len() < 3`) → 1 Hangul char = 3 UTF-8 bytes misrouted.
Double-quote wrapping forces exact phrase matching, blocking substring.

## Architecture (target state)

```
documents (NFC-normalized at rest)
   ├── trg_fts_* → documents_fts        (trigram, external-content)  [≥3-char, all scripts]
   └── trg_bigram_* → documents_bigram_fts (contentless, unicode61)  [2-char CJK]

Query routing:  0→empty  |  1→LIKE  |  2+CJK→bigram MATCH  |  2+Latin→LIKE  |  ≥3→trigram MATCH
```

Phase 3 adds `documents_choseong_fts` (contentless, `choseong_bigrams_cjk()`) for 초성 queries.

---

## Phase 1 — Immediate fix (no schema change, ships now)

**Goal:** 2-char Korean works (via LIKE) + byte-bug fixed + NFC + test coverage.

| Task | File | What |
|---|---|---|
| T1.1 | `src/db/test_support.rs` (new `#[cfg(test)]`) | `setup_db()`, `make_doc()`, `insert_doc()` helpers |
| T1.2 | `src/db/fts_query.rs` RED tests | Test byte-bug, Korean routing, edge cases |
| T1.3 | `src/db/fts_query.rs` GREEN | `FtsQuery { Match, Like }`, `build_fts_query()` with `chars().count()`, dedup |
| T1.4 | `Cargo.toml` + `fts_query.rs` | `unicode-normalization = "0.1"`, `normalize_nfc()` |
| T1.5 | `search.rs` + `facets.rs` | Route through `build_fts_query`; Like→WHERE LIKE; Match→FTS MATCH |
| T1.6 | `tests/database.rs` | Integration: Korean 2/3/1-char, mixed, NFC, no-false-positive, English, trigger sync |

---

## Phase 2 — Bigram FTS5 (indexed 2-char CJK) — gate after Phase 1

| Task | What |
|---|---|
| T2.1 | RED test for bigram routing |
| T2.2 | `bigrams_cjk()` core logic + tests |
| T2.3 | Register as SQL function (add `functions` feature to rusqlite) |
| T2.4 | NFC-normalize documents at rest |
| T2.5 | Migration v3: bigram table + triggers + NFC backfill + populate |
| T2.6 | Route 2-char CJK via `SearchPlan::BigramMatch` |
| T2.7 | Regression tests |

---

## Phase 3 — 초성 Search (bonus) — gate after Phase 2

| Task | What |
|---|---|
| T3.1 | Inline Hangul choseong decomposition + `choseong_bigrams_cjk()` |
| T3.2 | Register fn + migration v4 `documents_choseong_fts` + triggers |
| T3.3 | Route all-choseong queries |
| T3.4 | Docs |

---

## Rollback

- Phase 1: git revert, zero DB impact
- Phase 2: git revert + DROP TABLE/TRIGGER (additive only)
- Phase 3: same pattern
