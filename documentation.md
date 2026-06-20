# Libran — 기술 설계 문서 (Technical Design Document)

> **상태**: 구현 전 최종 설계 명세
> **날짜**: 2026-06-20
> **버전**: 1.0
> **기반**: 원본 제안서 리뷰 + 사용자 피드백 + 1차 출처 검증

---

## 1. 프로젝트 개요

### 1.1 정체성

Libran은 Rust 기반 **오프라인-first 서지 관리 시스템**이다. 핵심 원칙:

- **자체 연산 우선**: 모든 메타데이터 추출은 로컬 PDF 파싱으로 수행. AI/LLM에 의존하지 않는다.
- **오프라인 동작**: 외부 API 없이도 핵심 기능(분류, 검색, 관리, 내보내기)이 완전히 동작한다. API는 옵션.
- **CUI(문자 사용자 인터페이스)**: 터미널 듀얼 패널 인터페이스. GUI 플러그인(Word 등) 의존 배제.
- **다중 분류 스킴**: UDC Summary를 메인으로 하고, PhySH/MSC2020/LCC/사용자 정의를 보조로 통합.
- **다플랫폼**: macOS(Apple Silicon), Linux(x86_64/aarch64 musl 정적), Windows(x86_64 MSVC) 바이너리 산출.
- **이식성**: 서지 데이터는 BibTeX(.bib) 및 CSL JSON으로 내보내기. 원본 소유권은 사용자에게.

### 1.2 README 정합성

본 설계는 `README.md`에 명시된 방침을 최우선으로 따른다:

- UDC(국제십진분류법)를 공식 분류 체계로 채택 — README 7-16행 명시
- 분석합성식(`:` 결합, `-` 보조기호) 다학제 표현 지원 — README 핵심 가치
- "각 분류 체계를 UDC에 대응하되, 기존의 체계(MSC, PhySH, CAS)를 통한 검색 또한 지원" — README 16행

### 1.3 원본 제안서 대비 변경 사항

| 항목 | 제안서 | 본 설계 | 근거 |
|---|---|---|---|
| 분류 체계 | KDC 단독 | UDC Summary 메인 + 다중 보조 | README 방침 준수, KDC는 README와 충돌 |
| PDF 파서 | pdf_oxide 단독 | lopdf(메타) + unpdf(본문) 분리 | pdf_oxide 자가보고 성능수치 미검증, CJK/암호화 지원 |
| TLS | OpenSSL 정적 | rustls(순수 Rust) | 크로스컴파일 병목 제거 |
| FTS5 토크나이저 | unicode61 | trigram | CJK 부분매칭 지원 |
| 빌드 타깃 | x86_64만 | ARM/Apple Silicon 포함 4타깃 | 현대 연구자 노트북 환경 |
| API 제한 | 초당 10회 통일 | 유형별(10/s 단일, 3/s 목록) | CrossRef 2025.12 변경 반영 |
| 채널 | 무한 mpsc | 바운드 mpsc | 메모리 무증식 방지 |
| FTS5 테이블 | 독립 중복 저장 | 외부 콘텐츠 테이블 | 용량 절반 이하 |
| DOI 정규식 | prose-코드 불일치 | CrossRef 공식 패턴 정확 구현 | 출처 검증 |

---

## 2. 시스템 아키텍처

### 2.1 계층 구조

```
┌─────────────────────────────────────────────────────────┐
│                  UI 렌더링 계층                          │
│  Ratatui 듀얼 패널 (좌: 분류 트리+프로젝트 / 우: 문헌 리스트)│
│  crossterm 이벤트 루프 (브래킷 페이스트, Kitty 폴백)       │
└──────────────────────────▲──────────────────────────────┘
                           │ (상태 렌더링)
                           ▼
┌─────────────────────────────────────────────────────────┐
│                  비동기 제어 계층                         │
│  tokio mpsc 바운드 채널 액션 디스패처                     │
│  AppAction enum → 상태 변환 → UI 갱신                    │
└──────────────────────────▲──────────────────────────────┘
                           │ (비동기 태스크 스폰)
                           ▼
┌─────────────────────────────────────────────────────────┐
│               파일 및 연산 계층                          │
│  PDF 파싱(lopdf+unpdf) │ DOI/arXiv 추출 │ API 클라이언트  │
│  분류 추천 엔진 │ 인용키 생성 │ 내보내기                  │
└──────────────────────────▲──────────────────────────────┘
                           │ (영속 데이터 저장)
                           ▼
┌─────────────────────────────────────────────────────────┐
│                  데이터 스토리지                          │
│  SQLite (rusqlite, bundled)                              │
│  FTS5 trigram 외부콘텐츠 테이블 │ M:N 프로젝트 매핑       │
│  분류 스킴 DB (UDC/PhySH/MSC/LCC) │ 설정 저장             │
└─────────────────────────────────────────────────────────┘
```

### 2.2 모듈 구조

```
libran/
├── Cargo.toml                    # 프로젝트 매니페스트
├── Cross.toml                    # cross 크로스컴파일 설정
├── .cargo/
│   └── config.toml               # 타깃별 링커 매핑
├── build.rs                      # 빌드 스크립트 (분류 데이터 변환)
├── data/                         # 번들 분류 원본 데이터
│   ├── udcs.ttl                  # UDC Summary RDF (Finto 미러, 1.5MB)
│   ├── physh/                    # PhySH SKOS (GitHub)
│   ├── msc2020/                  # MSC2020 SKOS/CSV
│   └── lcc/                      # LCC MADS/RDF
├── assets/
│   └── udc_top_ko.csv            # UDC 최상위 34개 한국어 번역 테이블
├── src/
│   ├── main.rs                   # 진입점, tokio 런타임 가동
│   ├── app/
│   │   ├── mod.rs                # AppState, 액션 디스패처
│   │   ├── action.rs             # AppAction enum 정의
│   │   └── state.rs              # 전역 상태 구조체
│   ├── ui/
│   │   ├── mod.rs                # Ratatui 메인 렌더링
│   │   ├── layout.rs             # 듀얼 패널 레이아웃
│   │   ├── left_panel.rs         # 분류 트리 + 프로젝트 리스트
│   │   ├── right_panel.rs        # 문헌 리스트 + 상세
│   │   ├── status_bar.rs         # 상태 표시줄
│   │   └── theme.rs              # 색상/스타일
│   ├── terminal/
│   │   ├── mod.rs                # 터미널 초기화/복구
│   │   ├── input.rs              # crossterm 이벤트 처리
│   │   ├── paste.rs              # 브래킷 페이스트 모드 제어
│   │   └── drag_drop.rs          # 드래그 앤 드롭 경로 파싱
│   ├── pdf/
│   │   ├── mod.rs                # PDF 처리 파이프라인 조율
│   │   ├── metadata.rs           # lopdf 기반 XMP/카탈로그 메타 추출
│   │   ├── text.rs               # unpdf 기반 본문 텍스트 추출
│   │   ├── identifiers.rs        # DOI/arXiv ID 정규식 추출
│   │   └── heuristic.rs          # SciPlore Xtract 폰트 기반 제목 추정
│   ├── api/
│   │   ├── mod.rs                # API 모드 관리 (4모드 토글)
│   │   ├── crossref.rs           # CrossRef Polite Pool 클라이언트
│   │   ├── arxiv.rs              # arXiv API 클라이언트
│   │   ├── rate_limiter.rs       # 유형별 백오프 (2025.12 제한)
│   │   └── cache.rs              # 로컬 디스크 캐시 (TTL 30일)
│   ├── classification/
│   │   ├── mod.rs                # 다중 스킴 추상 트레이트
│   │   ├── scheme.rs             # ClassificationScheme trait
│   │   ├── udc.rs                # UDC Summary 파서/트리
│   │   ├── physh.rs              # PhySH 파서/트리
│   │   ├── msc.rs                # MSC2020 파서/트리
│   │   ├── lcc.rs                # LCC 파서/트리
│   │   ├── custom.rs             # 사용자 정의 스킴 로더
│   │   ├── recommender.rs        # 자동 분류 추천 엔진
│   │   └── label_override.rs     # 라벨 오버라이드 파일 (LLM 번역 여유)
│   ├── db/
│   │   ├── mod.rs                # 데이터베이스 연결 관리
│   │   ├── schema.rs             # 스키마 정의 (DDL)
│   │   ├── migrations.rs         # 마이그레이션
│   │   ├── documents.rs          # documents CRUD
│   │   ├── projects.rs           # projects + project_documents CRUD
│   │   ├── search.rs             # FTS5 trigram 검색 쿼리
│   │   └── facets.rs             # 패싯 집계 쿼리 (CTE)
│   ├── citation/
│   │   ├── mod.rs                # 인용키 생성 조율
│   │   ├── key_generator.rs      # 4모드 키 생성 (기본/제목/해시/커스텀)
│   │   ├── bibtex.rs             # .bib 내보내기
│   │   └── csl_json.rs           # CSL JSON 내보내기
│   ├── storage/
│   │   ├── mod.rs                # 파일 보관 정책 관리
│   │   ├── library.rs            # 라이브러리 폴더 관리
│   │   └── trash.rs              # 원본 휴지통 이동 (플랫폼별)
│   ├── config/
│   │   ├── mod.rs                # 설정 로드/저장
│   │   └── defaults.rs           # 기본값
│   └── export/
│       └── mod.rs                # 내보내기 파이프라인
├── tests/
│   ├── pdf_parsing.rs            # PDF 파싱 통합 테스트
│   ├── classification.rs         # 분류 스킴 테스트
│   ├── search.rs                 # FTS5 검색 테스트
│   ├── citation_keys.rs          # 인용키 생성 테스트
│   └── cross_platform.rs         # 경로 처리 크로스플랫폼 테스트
└── benches/
    └── pdf_bench.rs              # PDF 파싱 벤치마크
```

---

## 3. 데이터베이스 스키마

### 3.1 엔진

- **SQLite** via `rusqlite` 크레이트, `features=["bundled"]` (SQLite 소스를 정적 컴파일)
- FTS5 확장 활성화 (SQLite amalgamation에 포함, `SQLITE_ENABLE_FTS5`)
- 데이터베이스 파일 위치: `~/.libran/libran.db` (구성 가능)

### 3.2 테이블 정의

#### 3.2.1 서지 마스터 테이블

```sql
CREATE TABLE IF NOT EXISTS documents (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    title           TEXT NOT NULL,
    authors         TEXT,          -- JSON 배열: ["Smith, J.", "Kim, D."]
    journal         TEXT,
    pub_year        INTEGER,
    doi             TEXT UNIQUE,
    arxiv_id        TEXT UNIQUE,
    abstract        TEXT,
    keywords        TEXT,          -- JSON 배열
    -- 다중 스킴 분류 (하나의 문헌이 여러 스킴 코드를 가질 수 있음)
    -- 주 분류는 documents 테이블에, 상세는 document_classifications 테이블에
    file_path       TEXT,          -- 라이브러리 내 상대경로 또는 절대경로
    file_hash       TEXT,          -- SHA-256, 중복 검증용
    citation_key    TEXT UNIQUE,   -- BibTeX 키 (Smith2024 등)
    source          TEXT,          -- 'pdf_extract' | 'crossref' | 'arxiv' | 'manual'
    created_at      TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at      TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_documents_doi ON documents(doi);
CREATE INDEX IF NOT EXISTS idx_documents_arxiv ON documents(arxiv_id);
CREATE INDEX IF NOT EXISTS idx_documents_year ON documents(pub_year);
CREATE INDEX IF NOT EXISTS idx_documents_citation_key ON documents(citation_key);
```

#### 3.2.2 분류 코드 테이블 (다중 스킴 지원)

```sql
-- 분류 스킴 마스터
CREATE TABLE IF NOT EXISTS classification_schemes (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    code            TEXT UNIQUE NOT NULL,   -- 'udc', 'physh', 'msc', 'lcc', 'custom'
    name            TEXT NOT NULL,
    version         TEXT,                   -- '2026_1', 'v2.8.0', '2020' 등
    enabled         BOOLEAN DEFAULT 1,      -- 사용자 토글
    is_primary      BOOLEAN DEFAULT 0,      -- UDC가 primary
    license         TEXT,                   -- 라이선스 정보
    source_url      TEXT,                   -- 데이터 출처
    imported_at     TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 분류 노드 (각 스킴의 계층 트리)
CREATE TABLE IF NOT EXISTS classification_nodes (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    scheme_id       INTEGER NOT NULL,
    notation        TEXT NOT NULL,          -- '005', '517.9', 'SQ 5100' 등
    pref_label      TEXT NOT NULL,          -- 기본 라벨 (영어)
    alt_label       TEXT,                   -- 대체 라벨
    scope_note      TEXT,
    parent_id       INTEGER,                -- 계층 구조 (self-reference)
    sort_order      INTEGER DEFAULT 0,
    FOREIGN KEY (scheme_id) REFERENCES classification_schemes(id) ON DELETE CASCADE,
    FOREIGN KEY (parent_id) REFERENCES classification_nodes(id) ON DELETE CASCADE,
    UNIQUE(scheme_id, notation)
);

CREATE INDEX IF NOT EXISTS idx_nodes_scheme ON classification_nodes(scheme_id);
CREATE INDEX IF NOT EXISTS idx_nodes_parent ON classification_nodes(parent_id);
CREATE INDEX IF NOT EXISTS idx_nodes_notation ON classification_nodes(notation);

-- 라벨 오버라이드 (사용자/LLM 번역)
CREATE TABLE IF NOT EXISTS classification_labels (
    node_id         INTEGER NOT NULL,
    lang            TEXT NOT NULL,          -- 'ko', 'ja', 'zh' 등
    label           TEXT NOT NULL,
    source          TEXT,                   -- 'user' | 'llm' | 'community'
    created_at      TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (node_id) REFERENCES classification_nodes(id) ON DELETE CASCADE,
    PRIMARY KEY (node_id, lang)
);

-- 문헌-분류 매핑 (M:N, 한 문헌이 여러 스킴의 여러 코드에 매핑 가능)
CREATE TABLE IF NOT EXISTS document_classifications (
    document_id     INTEGER NOT NULL,
    node_id         INTEGER NOT NULL,
    is_primary      BOOLEAN DEFAULT 0,      -- 주 분류 여부
    confidence      REAL,                   -- 추천 신뢰도 (0.0~1.0), 수동 지정시 NULL
    assigned_by     TEXT,                   -- 'auto' | 'user'
    assigned_at     TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (document_id) REFERENCES documents(id) ON DELETE CASCADE,
    FOREIGN KEY (node_id) REFERENCES classification_nodes(id) ON DELETE CASCADE,
    PRIMARY KEY (document_id, node_id)
);

CREATE INDEX IF NOT EXISTS idx_doc_class_doc ON document_classifications(document_id);
CREATE INDEX IF NOT EXISTS idx_doc_class_node ON document_classifications(node_id);
```

#### 3.2.3 프로젝트 관리 테이블 (M:N)

```sql
CREATE TABLE IF NOT EXISTS projects (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    name            TEXT NOT NULL UNIQUE,
    description     TEXT,
    created_at      TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS project_documents (
    project_id      INTEGER NOT NULL,
    document_id     INTEGER NOT NULL,
    added_at        TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (project_id, document_id),
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
    FOREIGN KEY (document_id) REFERENCES documents(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_proj_docs_project ON project_documents(project_id);
CREATE INDEX IF NOT EXISTS idx_proj_docs_doc ON project_documents(document_id);
```

#### 3.2.4 FTS5 전문 검색 (trigram, 외부 콘텐츠 테이블)

```sql
-- 외부 콘텐츠 테이블: FTS 인덱스만 저장, 본문은 documents 테이블 참조
-- 용량 절반 이하, CJK 부분매칭 지원
CREATE VIRTUAL TABLE IF NOT EXISTS documents_fts USING fts5(
    title,
    authors,
    journal,
    abstract,
    keywords,
    content='documents',
    content_rowid='id',
    tokenize='trigram'
);

-- 외부 콘텐츠 동기화 트리거
CREATE TRIGGER IF NOT EXISTS trg_fts_insert AFTER INSERT ON documents BEGIN
    INSERT INTO documents_fts(rowid, title, authors, journal, abstract, keywords)
    VALUES (new.id, new.title, new.authors, new.journal, new.abstract, new.keywords);
END;

CREATE TRIGGER IF NOT EXISTS trg_fts_delete AFTER DELETE ON documents BEGIN
    INSERT INTO documents_fts(documents_fts, rowid, title, authors, journal, abstract, keywords)
    VALUES ('delete', old.id, old.title, old.authors, old.journal, old.abstract, old.keywords);
END;

CREATE TRIGGER IF NOT EXISTS trg_fts_update AFTER UPDATE ON documents BEGIN
    INSERT INTO documents_fts(documents_fts, rowid, title, authors, journal, abstract, keywords)
    VALUES ('delete', old.id, old.title, old.authors, old.journal, old.abstract, old.keywords);
    INSERT INTO documents_fts(rowid, title, authors, journal, abstract, keywords)
    VALUES (new.id, new.title, new.authors, new.journal, new.abstract, new.keywords);
END;
```

> **trigram 토크나이저 선택 근거**: CJK 텍스트는 공백으로 단어가 구분되지 않아 `unicode61`이 전체 문자열을 단일 토큰으로 처리하여 부분매칭이 불가능하다. `trigram`은 3문자 슬라이딩 윈도우로 부분문자열 매칭을 제공하여 "방정식" 검색이 "미분방정식해석학" 색인에서命中한다. 단점(3문자 미만 검색 불가, 인덱스 용량 원문의 ~3배)은 용인.

#### 3.2.5 API 응답 캐시 (TTL 30일)

```sql
CREATE TABLE IF NOT EXISTS api_cache (
    cache_key       TEXT PRIMARY KEY,       -- 식별자 또는 쿼리 해시
    source          TEXT NOT NULL,          -- 'crossref' | 'arxiv'
    response_json   TEXT NOT NULL,
    fetched_at      TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    expires_at      TIMESTAMP NOT NULL      -- fetched_at + 30일
);

CREATE INDEX IF NOT EXISTS idx_cache_expires ON api_cache(expires_at);
```

#### 3.2.6 설정 저장

```sql
CREATE TABLE IF NOT EXISTS app_config (
    key             TEXT PRIMARY KEY,
    value           TEXT,                   -- JSON 값
    updated_at      TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

### 3.3 패십 집계 쿼리 (CTE 최적화)

사용자가 검색어 입력 + 프로젝트 선택 시, 각 분류 코드별 매칭 건수를 단일 패스로 추출:

```sql
WITH ScopedDocuments AS (
    SELECT d.id, dc.node_id, cn.notation, cn.pref_label, cs.code as scheme_code
    FROM documents d
    INNER JOIN project_documents pd ON d.id = pd.document_id
    INNER JOIN document_classifications dc ON d.id = dc.document_id
    INNER JOIN classification_nodes cn ON dc.node_id = cn.id
    INNER JOIN classification_schemes cs ON cn.scheme_id = cs.id
    WHERE pd.project_id = :project_id
      AND d.id IN (SELECT rowid FROM documents_fts WHERE documents_fts MATCH :search_term)
      AND cs.enabled = 1
)
SELECT
    scheme_code,
    notation,
    pref_label,
    COUNT(id) AS facet_count
FROM ScopedDocuments
GROUP BY scheme_code, notation, pref_label
ORDER BY facet_count DESC, notation ASC;
```

---

## 4. 분류 체계 시스템

### 4.1 아키텍처

```
┌─────────────────────────────────────────────┐
│            ClassificationScheme trait        │
│  - notation_pattern() -> Regex               │
│  - parse_node(notation) -> Node              │
│  - children(parent) -> Vec<Node>             │
│  - ancestors(node) -> Vec<Node>              │
│  - search(term, lang) -> Vec<Node>           │
│  - validate(notation) -> bool                │
│  - label(node, lang) -> String               │
└──────────▲──────────────────────────────────┘
           │
  ┌────────┼────────┬────────┬────────┐
  │        │        │        │        │
┌─┴──┐  ┌─┴──┐  ┌─┴──┐  ┌─┴──┐  ┌─┴──────┐
│UDC │  │PhySH│  │MSC │  │LCC │  │Custom  │
│Sum.│  │    │  │2020│  │    │  │(loader)│
└────┘  └────┘  └────┘  └────┘  └────────┘
```

### 4.2 메인 스킴: UDC Summary

- **데이터 소스**: Finto 미러 (https://finto.fi/rest/v1/udcs/data?format=text/turtle)
- **라이선스**: CC BY-SA 3.0 (UDC Consortium 저작권, 저작자 표시 + 동일조건 공유)
- **규모**: 2,453개 컨셉 (SKOS Turtle, 1.5MB)
- **분류 방식**: 분석합성식 — `+`(조정), `/`(연속확장), `:`(관계결합), `-`(보조기호)가 데이터에 실제 정의됨
- **라벨 언어**: 영어(`@en`), 핀란드어(`@fi`), 스웨덴어(`@sv`)
- **한국어**: 없음 → 영어 기본 표시 + 라벨 오버라이드 시스템으로 점진 추가

**UDC 분석합성식 지원**:
- `+` 기호: "622+669 Mining and Metallurgy" (비연속 UDC 번호 결합)
- `:` 기호: "17:7 Ethics in relation to art" (관계 결합, README 예시 `51-72:517.9:538.91`의 핵심)
- `/` 기호: "629.734/.735" (연속 범위)
- `-` 기호: 보조 분류 (형식, 시간, 장소, 언어 등)

이 기호들은 UDC Summary RDF에 `skos:Concept`로 정의되어 있으며, 시스템은 이를 파싱하여 결합 표기 검증 및 트리 탐색에 활용한다.

### 4.3 보조 스킴

| 스킴 | 분야 | 라이선스 | 데이터 소스 | 규모 | 토글 |
|---|---|---|---|---|---|
| PhySH | 물리학 | CC0 | GitHub physh-org/PhySH (SKOS, v2.8.0) | ~3,000 컨셉 | 기본 활성화 |
| MSC2020 | 수학 | CC BY-NC-SA | msc2020.org (SKOS/CSV) | ~6,000 코드 | 기본 활성화 |
| LCC | 범용 | 공개 도메인 | id.loc.gov (MADS/RDF) | 대규모 | 기본 비활성화 |
| Custom | 사용자 정의 | 사용자 소유 | 로컬 파일 (JSON/CSV/SKOS) | 가변 | 사용자 추가 |

- 각 스킴은 독립적으로 번들링되며 `classification_schemes.enabled`로 토글.
- 모든 스킴은 UDC에 "대응" — 교차 매핑 테이블(`classification_crosswalks`)은 향후 추가 가능.
- 사용자는 한 문헌에 여러 스킴의 코드를 동시 부여 가능.

### 4.4 라벨 오버라이드 시스템 (LLM 번역 여유)

```
data/udc_top_ko.csv    ← 최상위 34개 한국어 번역 (자체 제작, ~70 라벨)
classification_labels  ← DB 테이블 (node_id, lang, label, source)
~/.libran/labels/      ← 사용자 오버라이드 파일 디렉토리
  udc_ko.json          ← 사용자/커뮤니티 한국어 라벨
  udc_ja.json          ← 일본어 라벨 (확장)
```

**라벨 해결 우선순위**:
1. `classification_labels` 테이블의 사용자 오버라이드 (lang 매칭)
2. `classification_labels`의 LLM 번역 (`source='llm'`)
3. 원본 `pref_label` (영어)
4. `notation` 자체 (라벨 없을 시 폴백)

**LLM 번역 확장점**:
- `label_override.rs`에 `translate_labels(scheme, target_lang, llm_config)` 함수 시그니처 예약
- LLM 설정은 `app_config`에 저장 (provider, model, api_key) — 단, 핵심 기능은 LLM 없이 동작
- 번역 결과는 `classification_labels`에 `source='llm'`으로 저장되어 영속
- 번역 품질 검토 후 `source='user'`로 승격 가능

### 4.5 자동 분류 추천 엔진

PDF에서 추출된 메타데이터(저널명, 키워드, 초록, 제목)를 기반으로 UDC 코드를 자동 추천:

1. **키워드 매칭**: 추출된 키워드를 각 분류 노드의 `pref_label`/`alt_label`/`scope_note`와 FTS5 검색
2. **저널명 매칭**: 저널명을 분류 노드 라벨과 유사도 비교
3. **초록 분석**: 초록 텍스트를 trigram FTS로 각 분류 영역과 교차 검색
4. **신뢰도 점수**: 매칭 빈도/가중치로 `confidence` (0.0~1.0) 산출
5. **상위 N개 추천**: 사용자가 CUI에서 확인 후 선택 또는 수동 지정

추천은 제안만 하고, 최종 분류는 항상 사용자가 확정한다.

---

## 5. PDF 파싱 파이프라인

### 5.1 엔진 조합

```
PDF 파일 입력
     │
     ▼
┌─────────────┐     ┌──────────────────┐
│   lopdf     │     │      unpdf       │
│  (메타데이터) │     │   (본문 텍스트)    │
│             │     │                  │
│ - XMP RDF   │     │ - 본문 텍스트     │
│ - 카탈로그   │     │ - CJK/RTL 지원   │
│   Title     │     │ - 다열 감지      │
│   Author    │     │ - 암호화 처리     │
│   Subject   │     │ - 스트리밍 파싱   │
│   Creator   │     │ - Rayon 병렬     │
│   Producer  │     │                  │
└──────┬──────┘     └────────┬─────────┘
       │                     │
       │    ┌────────────────┘
       ▼    ▼
┌──────────────────────────────┐
│       메타데이터 통합          │
│  XMP 우선 → 카탈로그 폴백      │
│  DOI/arXiv ID 정규식 추출      │
│  SciPlore 휴리스틱 제목 추정    │
└──────────────────────────────┘
```

### 5.2 lopdf 기반 메타데이터 추출

- **용도**: PDF 내부 구조(카탈로그, 트레일러, XMP 메타데이터 스트림) 저수준 파싱
- **추출 순서**:
  1. XMP 메타데이터 세그먼트 쿼리 (`Metadata` 객체, XML 타입)
  2. Dublin Core 네임스페이스 파싱: `dc:creator`, `dc:title`, `dc:description`, `dc:date`
  3. XMP 불완전 시 PDF 카탈로그 정보 딕셔너리 스캔: `Title`, `Author`, `Subject`, `Creator`, `Producer`, `CreationDate`, `ModDate`
- **비동기 격리**: `tokio::task::spawn_blocking` 내에서 실행 (CPU 집약적, 블로킹 I/O)

### 5.3 unpdf 기반 본문 텍스트 추출

- **용도**: 본문 텍스트 디코딩, CJK/RTL 폰트 처리, 다열 레이아웃 감지
- **기능 활용**:
  - CJK 스마트 스페이싱 (Adobe CMap 리소스)
  - RTL 텍스트 BiDi 재정렬 (아랍어/히브리어)
  - 다열 감지 (Recursive XY-Cut 알고리즘)
  - 암호화 PDF 자동 복호화 (빈 사용자 비밀번호 시)
  - Rayon 병렬 페이지 처리
  - 스트리밍 파싱 (`for_each_page`, 메모리 효율)
- **추출 범위**: 전체 텍스트 (DOI/arXiv 검출용) + 첫 2페이지 (제목/식별자 우선 검색)

### 5.4 어댑터 패턴 (교체 가능)

```rust
pub trait PdfMetadataExtractor {
    fn extract_metadata(&self, path: &Path) -> Result<RawMetadata>;
}

pub trait PdfTextExtractor {
    fn extract_text(&self, path: &Path, page_range: Option<Range<u32>>) -> Result<String>;
}
```

- `lopdf`와 `unpdf`는 각각 이 트레이트를 구현
- 향후 다른 파서로 교체 시 어댑터만 교체, 상위 로직 불변
- 사전-1.0 크레이트이므로 버전 고정 (`=0.x.y`)

### 5.5 DOI/arXiv ID 정규식

**DOI 정규식** (CrossRef 공식 패턴 기반, 출처: crossref.org/blog/dois-and-matching-regular-expressions):

```rust
// CrossRef 공식: 74.9M DOIs 중 74.4M 매칭 (99.3%)
// 네거티브 룩어헤드로 인용 경계 따옴표/꺽쇠 혼입 예방 (prose와 코드 일치)
static DOI_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b10\.\d{4,9}/[-._;()/:A-Z0-9]+[A-Z0-9]").unwrap()
});
```

- 정규화: 끝의 구두점(`.`, `;`, `,`) 제거
- 표시용 원본 보존 + 정규화 키 분리 저장
- 2008년 이전 레거시 DOI(`<>#+` 포함)는 별도 폴백 패턴으로 처리

**arXiv ID 정규식**:

```rust
// 신형(2007.04~): YYMM.NNNNN(vVersion) + 구형 전공 표기식 동시 감지
static ARXIV_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)\b(?:arxiv:)?(\d{4}\.\d{4,5}(?:v\d+)?)\b|\b(?:arxiv:)?([a-z\-]+(?:\.[a-z]+)?/\d{7}(?:v\d+)?)\b"
    ).unwrap()
});
```

### 5.6 SciPlore Xtract 휴리스틱

DOI/arXiv ID 검출 실패 시 폴백:

1. 첫 페이지 상단 문자열 군집의 폰트 크기 속성 분석
2. 마진 여백 레이아웃 정보 역컴파일
3. 상대적으로 가장 굵고 상위에 배치된 라인을 "논문 제목" 후보로 판독
4. 추정 제목 + 파일명을 기반으로 API 질의 파이프라인 작동 (옵션 모드에서만)

> **오프라인 동작 보장**: 식별자 검출 실패 + 오프라인 모드인 경우, 추정 제목과 파일명으로 메타데이터를 부분 구성하고 사용자가 수동 보완.

---

## 6. 외부 API 통합 (4모드 토글)

### 6.1 API 모드

```rust
pub enum ApiMode {
    /// 식별자(DOI/arXiv)가 검출된 경우만 직접 조회. 제목 검색 미사용.
    IdentifierOnly,
    /// 자동은 식별자 조회만. 사용자가 명시적 "온라인으로 찾기" 시에만 제목 검색.
    ManualSearch,
    /// 식별자 없으면 자동으로 query.bibliographic 제목 검색 폴백.
    AutoFallback,
    /// 외부 API 전면 배제. 모든 메타는 PDF 자체 추출만.
    FullyOffline,
}
```

- 기본값: `IdentifierOnly` (오프라인-first 원칙)
- 설정에서 사용자 토글 (`app_config`에 저장)
- 모드 변경 시 CUI 상태 표시줄에 즉시 반영

### 6.2 CrossRef Polite Pool 클라이언트

- **TLS**: `reqwest` with `rustls-tls` (OpenSSL 의존 배제)
- **Polite Pool 자격**: User-Agent 헤더에 `mailto:` 포함
  ```
  User-Agent: Libran/1.0 (mailto:{user_email})
  ```
- **이메일 입력**: 사용자 설정에서 관리, 미설정 시 공개 풀(Public Pool)로 동작 (제한 더 엄격)
- **제한 (2025.12 변경 반영)**:

| 요청 유형 | 엔드포인트 | Rate limit | Concurrency |
|---|---|---|---|
| 단일 DOI | `/works/{doi}` | 10/s | 3 |
| 목록 검색 | `/works?query.bibliographic=...` | 3/s | 3 |

- **백오프**: `X-Rate-Limit-Limit`/`X-Rate-Limit-Interval` 헤더 파싱, 429 수신 시 지수 백오프
- **캐시**: 응답은 `api_cache` 테이블에 저장 (TTL 30일), 동일 식별자 재호출 방지
- **User-Agent 안전 처리**: 이메일의 비-ASCII 문자 정제 (`.unwrap()` 배거, `?` 또는 검증 로직)

### 6.3 arXiv API 클라이언트

- **엔드포인트**: `http://export.arxiv.org/api/query?id_list={arxiv_id}`
- **응답 형식**: Atom XML
- **제한**: 3초 간격 권장 (arXiv API 에티켓)
- **파싱**: XML에서 제목, 저자, 초록, 주제 분류 추출

### 6.4 레이트 리미터

```rust
pub struct RateLimiter {
    mode: ApiMode,
    // 유형별 카운터
    single_doi_counter: TokenBucket,    // 10/s
    list_query_counter: TokenBucket,    // 3/s
    // 동시성 제어
    semaphore: Arc<Semaphore>,          // max 3 동시
    // 백오프 상태
    backoff_until: Option<Instant>,
}
```

- 토큰 버킷 알고리즘으로 유형별 독립 카운팅
- 429 수신 시 해당 유형의 백오프 활성화 (지수: 1s → 2s → 4s → 8s, 최대 60s)
- 백오프 중 해당 유형 요청은 즉시 캐시 확인 또는 실패 반환

---

## 7. 터미널/CUI 시스템

### 7.1 터미널 초기화

```rust
pub fn setup_terminal() -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        EnableBracketedPaste,           // 드래그 앤 드롭 수신
        EnableMouseCapture,             // (선택) 마우스 스크롤
        PushKeyboardEnhancementFlags(   // Kitty 프로토콜 시도
            KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
                | KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPES
                | KeyboardEnhancementFlags::REPORT_ALTERNATE_KEYS
                | KeyboardEnhancementFlags::REPORT_EVENT_TYPES
        ),
    )?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}
```

- `PushKeyboardEnhancementFlags`는 지원 터미널(iTerm2, Kitty, WezTerm 등)에서만 동작
- 미지원 터미널에서는 자동 폴백 (에러 무시, 단일 Key Press 폴링)
- 종료 시 `DisableBracketedPaste`, `PopKeyboardEnhancementFlags`, `LeaveAlternateScreen`, `disable_raw_mode` 순차 복구

### 7.2 브래킷 페이스트 모드

- `EnableBracketedPaste` 활성화 시 터미널이 붙여넣기/드래그앤드롭 텍스트를 `\e[200~` ... `\e[201~`로 감싸 전송
- crossterm은 이를 `Event::Paste(String)` 이벤트로 정제하여 반환
- 비-BPM 터미널의 이스케이프 공백(`\ `) 보정은 파싱 단계에서 처리

### 7.3 드래그 앤 드롭 경로 파싱

```rust
pub fn parse_dragged_path(input: &str) -> Option<PathBuf> {
    let trimmed = input.trim();
    if trimmed.is_empty() { return None; }

    let mut path_str = trimmed.to_string();

    // 1. 따옴표 제거 (운영체제/터미널별 자동 삽입)
    strip_surrounding_quotes(&mut path_str);

    // 2. 비-BPM 역슬래시 이스케이프 공백 보정
    //    /Users/academic/My\ Document.pdf -> /Users/academic/My Document.pdf
    path_str = path_str.replace("\\ ", " ");

    let path = PathBuf::from(path_str);

    // 3. 파일 존재 확인 (확장자 무관 — 서지 관리는 .bib, .tex, .html 등도 입력 가능)
    if path.exists() && path.is_file() {
        return Some(path);
    }
    None
}
```

> **제안서 대비 변경**: PDF만 허용하던 제약 제거. 서지 관리 시스템은 BibTeX(.bib), 논문 원문(.tex), preprint(.html), ePub 등도 색인 대상이므로 확장자 무관 수용. 후속 파이프라인에서 확장자별 분기.

### 7.4 Kitty 키보드 프로토콜 폴백

- POSIX 터미널은 기본적으로 Key Release를 감지하지 않음
- Kitty 프로토콜(`PushKeyboardEnhancementFlags`)이 활성화된 터미널에서만 Key Release 지원
- 미지원 터미널: 단일 Key Press 폴링 기반 토글 제어 (누름/해제 구분 없이 누름 이벤트로 상태 전환)
- 크로스 플랫폼 차원에서 토글 인터페이스로 설계 — 두 경우 모두 동일한 사용자 경험

### 7.5 듀얼 패널 레이아웃

```
┌──────────────────────┬───────────────────────────────────┐
│  Libran              │  문헌 리스트                       │
│  ─────────────       │  ─────────────────────────────     │
│  ▸ 프로젝트           │  ▸ [UDC 005] Nonisothermal...     │
│    • 머신러닝 가속     │    Smith, J. (2024)               │
│    • CUI 렌더러 설계   │    doi:10.33048/SIBJIM.2022.25.103│
│                      │  ▸ [UDC 517.9] Differential...     │
│  ▸ 분류 (UDC)         │    Kim, D. (2023)                 │
│    0 총류             │  ▸ [MSC 35-XX] PDE solutions...    │
│    5 자연과학          │    Lee, S. (2023)                 │
│    ▸ 51 수학           │                                   │
│      ▸ 517 해석학      │  ─────────────────────────────    │
│        517.9 미분방정식 │  [Space] 선택  [o] 온라인 조회     │
│    ▸ 53 물리학          │  [e] 편집  [d] 삭제  [x] 내보내기  │
│  ▸ 분류 (PhySH)       │                                   │
│    ...                │                                   │
│  ─────────────       │                                   │
│  [Tab] 패널 이동       │                                   │
│  [n] 새 프로젝트       │                                   │
└──────────────────────┴───────────────────────────────────┘
└─────────────── 상태 표시줄 ────────────────────────────────┘
 준비됨 | API: 식별자만 | 42 문헌 | 오프라인
```

- **좌측 패널**: 프로젝트 리스트 + 활성화된 분류 스킴 트리 (패싯 카운트 표시)
- **우측 패널**: 선택된 카테고리/프로젝트의 문헌 리스트
- **Tab**: 패널 간 포커스 이동
- **j/k 또는 방향키**: 리스트 탐색
- **Space**: 다중 선택 마킹
- **상태 표시줄**: 현재 API 모드, 문헌 수, 온라인/오프라인 상태, 처리 중 메시지

### 7.6 이벤트 루프

```rust
// 메인 루프 (tokio 런타임 내)
loop {
    // 1. 터미널 이벤트 폴링 (crossterm, 비동기)
    if event::poll(timeout)? {
        match event::read()? {
            Event::Key(key) => dispatch_action(AppAction::KeyPressed(key)),
            Event::Paste(text) => {
                if let Some(path) = parse_dragged_path(&text) {
                    dispatch_action(AppAction::DragDetected(path));
                }
            }
            Event::Resize(w, h) => dispatch_action(AppAction::Resized(w, h)),
            _ => {}
        }
    }

    // 2. 비동기 액션 채널 수신 (백그라운드 태스크 결과)
    if let Ok(action) = action_rx.try_recv() {
        dispatch_action(action);
    }

    // 3. 상태 변경 시 UI 렌더링
    if state.is_dirty() {
        terminal.draw(|frame| ui::render(frame, &state))?;
        state.clear_dirty();
    }
}
```

---

## 8. 비동기 아키텍처

### 8.1 액션 디스패처 (바운드 채널)

```rust
#[derive(Clone, Debug)]
pub enum AppAction {
    Tick,
    KeyPressed(KeyEvent),
    Resized(u16, u16),
    DragDetected(PathBuf),
    StartMetadataExtraction(PathBuf),
    MetadataExtracted(Box<RawMetadata>),
    StartApiLookup(String),              // DOI 또는 arXiv ID
    ApiLookupSuccess(Box<BibliographicRecord>),
    ApiLookupFailed(String),
    ApiLookupSkipped(String),            // 오프라인 모드로 스킵
    UpdateSearchFilter(String),
    SelectProject(Option<i64>),
    ToggleClassificationScheme(String),
    AssignClassification(i64, i64, bool),// doc_id, node_id, is_primary
    ExportBibtex(Vec<i64>, PathBuf),
    ExportCslJson(Vec<i64>, PathBuf),
    OperationFailed(String),
    SystemShutdown,
}

pub struct AppState {
    pub active_project_id: Option<i64>,
    pub document_search_term: String,
    pub is_processing: bool,
    pub status_text: String,
    pub api_mode: ApiMode,
    pub action_tx: mpsc::Sender<AppAction>,  // 바운드 (백프레셔)
    // ... UI 상태
}
```

- **바운드 채널**: `mpsc::channel(256)` — 대량 PDF 일괄 투하 시 생산이 소비를 압도하면 송신자가 대기 (백프레셔)
- 무한 채널(`UnboundedSender`) 배거: 메모리 무증식 방지

### 8.2 백그라운드 태스크 격리

```rust
AppAction::DragDetected(path) => {
    state.is_processing = true;
    state.status_text = format!("파일 처리 중: {}", path.display());

    let tx = state.action_tx.clone();
    tokio::spawn(async move {
        // 1. 디스크 I/O + PDF 파싱: spawn_blocking (블로킹 격리)
        let result = tokio::task::spawn_blocking(move || {
            pdf::process_file(&path)
        }).await;

        match result {
            Ok(Ok(metadata)) => {
                let _ = tx.send(AppAction::MetadataExtracted(Box::new(metadata)));
            }
            Ok(Err(e)) => {
                let _ = tx.send(AppAction::OperationFailed(e.to_string()));
            }
            Err(e) => {
                let _ = tx.send(AppAction::OperationFailed(format!("태스크 실패: {}", e)));
            }
        }
    });
}
```

- **CPU 집약 작업** (PDF 파싱, 정규식): `spawn_blocking`으로 별도 스레드 풀에서 실행
- **네트워크 I/O** (API 호출): `tokio::spawn`으로 비동기 런타임에서 실행
- **DB 쓰기**: `spawn_blocking` 내에서 `rusqlite` 동기 API 호출
- 메인 이벤트 루프는 어떠한 블로킹도 수행하지 않음 → UI 프레임 드랍 방지

### 8.3 런타임 진입

```rust
#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> Result<()> {
    // 터미널 초기화, DB 연결, 분류 데이터 로드
    // ...
    // 메인 루프 (이벤트 루프 + 채널 수신)
    // ...
    // 정리: 터미널 복구, DB 커밋
}
```

- `tokio::spawn`은 런타임 컨텍스트 내에서만 유효 — `#[tokio::main]` 보장
- 워커 스레드 4개 (PDF/API/DB/UI 렌더링 병렬)

---

## 9. 인용키 생성 시스템

### 9.1 4모드 지원

```rust
pub enum CitationKeyMode {
    /// 성+연도 + 중복 접미사(a,b,c). 기본값.
    /// 다국어: 한글/중문/일문 저자는 성을 그대로 (Kim2024, 田中2024)
    AuthorYear,
    /// 성+연도+제목첫단어 (Smith2024Nonisothermal)
    AuthorYearTitle,
    /// 성+연도+단축해시 (Smith2024a3f). 충돌 0 보장.
    AuthorYearHash,
    /// 사용자 커스텀 템플릿 ({author}{year}{titleword})
    Custom(String),  // 템플릿 문자열
}
```

### 9.2 기본 모드 (AuthorYear) 상세

1. 첫 번째 저자의 성 추출:
   - 서양 이름: 마지막 단어 (Smith, Johnson)
   - 동아시아 이름: 첫 글자/단어 (Kim, 田中, 李) — 성이 앞에 옴
   - 다국어 보존: 원본 문자 그대로 (로마자 변환 강제 안 함)
2. 출판 연도 추출 (메타데이터에서, 없으면 "n.d.")
3. 키 생성: `{성}{연도}` (예: `Smith2024`)
4. 중복 확인 (DB에서 `citation_key` UNIQUE 제약):
   - 중복 시 접미사: `Smith2024a`, `Smith2024b`, `Smith2024c`...
   - 접미사 순서: a → b → c → ... → z → aa → ab...
5. 다국어 정규화: 키에 비-ASCII 허용 (BibTeX은 ASCII 권장이나 현대 처리기는 UTF-8 지원)
   - 사용자 설정으로 ASCII 변환 옵션 (Kim2024, Tanaka2024 등 로마자화 토글)

### 9.3 커스텀 템플릿 모드 (심혈 기울여 구현)

```rust
// 템플릿 변수:
//   {author}     — 첫 저자 성
//   {author_full}— 첫 저자 전체 이름
//   {year}       — 출판 연도 (4자리)
//   {year2}      — 출판 연도 (2자리)
//   {title}      — 제목 (공백 제거, 첫 N문자)
//   {titleword}  — 제목 첫 단어 (불용어 제거)
//   {journal}    — 저널명 약어
//   {doi_suffix} — DOI 접미사 (10.xxxx/ 뒤 부분)
//   {hash6}      — 메타데이터 해시 앞 6자리
//   {type}       — 문헌 유형 (article, book, etc.)

// 템플릿 예시:
//   "{author}{year}{titleword}"  -> Smith2024Nonisothermal
//   "{author}_{year}_{hash6}"    -> Smith_2024_a3f2b1
//   "{year}/{author}{doi_suffix}" -> 2024/SmithSIBJIM2022.25.103
```

**구현 세부**:
- 템플릿 파서: 중괄호 `{}` 내 변수명 추출, 나머지는 리터럴
- 변수별 변환 함수 등록 (HashMap<&str, fn(&Metadata) -> String>)
- 불용어 목록 (the, a, an, of, in, on, for, and — 영어; 의, 를, 에, 에서 — 한국어)
- 제목 단어 추출 시 불용어 건너뜀
- 생성된 키의 유효성 검증 (BibTeX 키 규칙: 영숫자 + 일부 기호)
- 검증 실패 시 자동 정제 (무효 문자 제거/대체)
- 중복 시 접미사 추가 (다른 모드와 동일 로직)

### 9.4 키 충돌 해결 (공통)

```rust
pub fn resolve_collision(base_key: &str, exists: impl Fn(&str) -> bool) -> String {
    if !exists(base_key) {
        return base_key.to_string();
    }
    // a, b, c, ..., z, aa, ab, ...
    for suffix in SuffixIterator::new() {
        let candidate = format!("{}{}", base_key, suffix);
        if !exists(&candidate) {
            return candidate;
        }
    }
    // 극단적 충돌 시 해시 추가
    format!("{}_{}", base_key, short_hash())
}
```

---

## 10. 내보내기 시스템

### 10.1 BibTeX (.bib)

```bibtex
@article{Smith2024,
  author    = {Smith, J. and Kim, D.},
  title     = {Nonisothermal diffuse interface model for electrical breakdown channel propagation},
  journal   = {Siberian Journal of Industrial Mathematics},
  year      = {2024},
  doi       = {10.33048/SIBJIM.2022.25.103},
  keywords  = {diffuse interface, electrical breakdown},
  file      = {:/Users/honey/.libran/library/Smith2024.pdf:PDF}
}
```

- 선택된 문헌(또는 전체)을 `.bib` 파일로 내보내기
- 필드 매핑: documents 테이블 → BibTeX 엔트리
- `file` 필드: 라이브러리 내 PDF 경로 (BibTeX 호환 참조)
- 인용키는 `citation_key` 컬럼값 사용

### 10.2 CSL JSON

```json
[
  {
    "id": "Smith2024",
    "type": "article-journal",
    "title": "Nonisothermal diffuse interface model...",
    "author": [
      {"family": "Smith", "given": "J."},
      {"family": "Kim", "given": "D."}
    ],
    "container-title": "Siberian Journal of Industrial Mathematics",
    "issued": {"date-parts": [[2024]]},
    "DOI": "10.33048/SIBJIM.2022.25.103",
    "keyword": "diffuse interface, electrical breakdown"
  }
]
```

- CSL( Citation Style Language) 호환 JSON 포맷
- Typst, LaTeX, pandoc, Zotero 등 외부 도구에서 직접 소비 가능
- 인용포맷 렌더링(Chicago, APA 등)은 외부 도구에 위임 — Libran은 렌더링하지 않음
- 이 접근은 제안서의 "Word 플러그인 배제" 철학과 일관됨

### 10.3 내보내기 경로

- CUI에서 `x` 키 → 내보내기 메뉴 → BibTeX / CSL JSON 선택
- 파일 저장 대화 (경로 입력)
- 프로젝트별 내보내기 (해당 프로젝트 문헌만)
- 전체 라이브러리 내보내기

---

## 11. 파일 보관 정책

### 11.1 3모드 토글

```rust
pub enum FileStoragePolicy {
    /// PDF를 라이브러리 폴더로 복사. 원본은 그대로.
    /// 무결성 최고, 용량 2배.
    CopyToLibrary,
    /// 원본 경로만 참조. 파일은原地.
    /// 용량 절약, 원본 이동/삭제 시 끊어짐.
    ReferenceOnly,
    /// 복사 후 원본을 휴지통으로 이동 (사용자 확인 포함).
    /// 단일 진실원, 용량 중복 회피.
    CopyAndTrash,
}
```

- 기본값: `CopyToLibrary`
- 설정에서 토글 (`app_config`)
- `CopyAndTrash` 모드는 파괴적이므로 확인 다이얼로그 필수

### 11.2 라이브러리 폴더

- 기본 위치: `~/.libran/library/`
- 파일명 규칙: `{citation_key}.pdf` (예: `Smith2024.pdf`)
- 중복 파일 감지: SHA-256 해시 (`file_hash` 컬럼)
- 동일 해시 시 신규 등록 거부 (또는 기존 문헌에 프로젝트만 추가)

### 11.3 원본 휴지통 이동 (플랫폼별)

- **macOS**: `mv` to `~/.Trash/` 또는 NSWorkspace recycleURLs (FFI)
- **Linux**: `trash-put` 명령 (trash-cli 패키지) 또는 `~/.local/share/Trash/`
- **Windows**: `SHFileOperation` with FO_DELETE + FOF_ALLOWUNDO (FFI) 또는 `trash` 크레이트
- 플랫폼별 모듈 분리 (`#[cfg(target_os = "...")]`)

### 11.4 끊어진 참조 복구

`ReferenceOnly` 모드에서 원본 이동/삭제 시:
- 파일 존재 여부 주기 검사 (또는 접근 시 검사)
- 끊어진 참조 시 CUI에 경고 표시
- "파일 재연결" 기능 — 사용자가 새 경로 지정

### 11.5 크로스플랫폼 경로 처리

- **절대 원칙**: 경로 조작 시 `PathBuf::join()`, `PathBuf::push()`만 사용. 하드코딩 슬래시(`/` 또는 `\`) 금지.
- 라인 엔딩: 설정 파일/내보내기 파일 쓰기 전 `\n`으로 정규화 (CRLF/LF 통일)
- 경로 저장: DB에는 운영체제 네이티브 경로 저장, 이식 시 변환 로직 제공

---

## 12. 크로스 플랫폼 빌드

### 12.1 타깃 매트릭스

| 타깃 | 플랫폼 | 용도 | 방식 |
|---|---|---|---|
| `aarch64-apple-darwin` | macOS Apple Silicon | 개발 네이티브, 배포 | 로컬 빌드 |
| `x86_64-unknown-linux-musl` | Linux x86_64 정적 | 배포 (glibc 무관) | `cross` (Docker) |
| `aarch64-unknown-linux-musl` | Linux ARM 정적 | Raspberry Pi/Graviton | `cross` (Docker) |
| `x86_64-pc-windows-msvc` | Windows x86_64 | 배포 | `cross` (Docker) |

### 12.2 Cargo.toml 핵심 의존성

```toml
[dependencies]
# CUI
ratatui = "0.x"
crossterm = { version = "0.x", features = ["event-stream"] }
# 비동기
tokio = { version = "1", features = ["full"] }
# PDF
lopdf = "0.x"
unpdf = "=0.x.y"           # 버전 고정 (사전-1.0)
# DB
rusqlite = { version = "0.x", features = ["bundled"] }  # SQLite 소스 정적 컴파일
# HTTP (rustls, OpenSSL 배제)
reqwest = { version = "0.x", default-features = false, features = ["rustls-tls", "json"] }
# 정규식
regex = "1"
once_cell = "1"
# 분류 데이터 파싱
rio_turtle = "0.x"          # Turtle RDF 파싱
oxrdf = "0.x"               # RDF 데이터 모델
quick-xml = "0.x"           # XML 파싱 (MSC, LCC MADS)
# 파일 해시
sha2 = "0.x"
# 직렬화
serde = { version = "1", features = ["derive"] }
serde_json = "1"
# 경로
directories = "5"           # 플랫폼별 표준 디렉토리
# 오류
thiserror = "1"
anyhow = "1"
```

### 12.3 .cargo/config.toml

```toml
# Apple Silicon 네이티브 (로컬 빌드)
# 별도 링커 설정 불필요 (Xcode 툴체인 사용)

# Linux/Windows 타깃은 cross 엔진 사용 (Docker 컨테이너)
# 로컬 직접 크로스컴파일은 glibc 헤더 문제로 실패하므로 cross 의존
```

### 12.4 Cross.toml

```toml
[target.x86_64-unknown-linux-musl]
image = "ghcr.io/cross-rs/x86_64-unknown-linux-musl:latest"

[target.aarch64-unknown-linux-musl]
image = "ghcr.io/cross-rs/aarch64-unknown-linux-musl:latest"

[target.x86_64-pc-windows-msvc]
image = "ghcr.io/cross-rs/x86_64-pc-windows-msvc:latest"

[build.env]
passthrough = [
    "SQLITE3_BUNDLED",   # rusqlite bundled 피처 정적 컴파일 보장
]
# OPENSSL_STATIC 제거 — rustls 사용으로 OpenSSL 의존 자체 배제
```

### 12.5 빌드 스크립트 (build.rs)

- 분류 원본 데이터(`data/*.ttl`, `data/*.rdf`)를 SQLite 테이블로 변환
- 변환된 DB는 바이너리에 임베드 (`include_bytes!`) 또는 첫 실행 시 생성
- PhySH/MSC/LCC 데이터도 동일하게 사전 변환
- 빌드 시점 변환으로 런타임 부담 제거

### 12.6 멀티플랫폼 무결성 수칙

1. **경로 조작**: `PathBuf::join()`/`push()`만 사용, 하드코딩 슬래시 금지
2. **라인 엔딩**: 파일 쓰기 전 `\n` 정규화
3. **플랫폼별 코드**: `#[cfg(target_os = "...")]`로 분리 (휴지통 이동 등)
4. **TLS**: rustls(순수 Rust)로 크로스컴파일 병목 제거
5. **SQLite**: `bundled` 피처로 소스 정적 컴파일, 시스템 SQLite 의존 배제

---

## 13. 설정 시스템

### 13.1 설정 항목

```rust
pub struct AppConfig {
    // API
    pub api_mode: ApiMode,                  // IdentifierOnly (기본)
    pub user_email: Option<String>,         // CrossRef Polite Pool용

    // 파일 보관
    pub file_storage_policy: FileStoragePolicy,  // CopyToLibrary (기본)
    pub library_path: PathBuf,              // ~/.libran/library/

    // 인용키
    pub citation_key_mode: CitationKeyMode,  // AuthorYear (기본)
    pub citation_key_template: Option<String>, // Custom 모드용
    pub citation_key_ascii_only: bool,       // false (UTF-8 허용)

    // 분류
    pub primary_scheme: String,              // "udc"
    pub enabled_schemes: Vec<String>,        // ["udc", "physh", "msc"]
    pub label_language: String,              // "en" (영어 기본)

    // 검색
    pub search_fuzzy: bool,                  // false (trigram 정확 매칭)

    // DB
    pub db_path: PathBuf,                    // ~/.libran/libran.db
}
```

### 13.2 설정 저장

- `app_config` 테이블 (key-value, JSON 값)
- 첫 실행 시 기본값으로 초기화
- CUI 설정 메뉴에서 편집
- 변경 시 즉시 DB에 저장

### 13.3 설정 파일 (외부)

- `~/.libran/config.toml` (선택적, DB 설정과 동기화)
- DB가 주 저장소, TOML은 사용자 편의용 내보내기/가져오기

---

## 14. 개발 마일스톤

### 마일스톤 1: 프로젝트 스캐폴딩 + DB 스키마 (1~2주차)

- Cargo 프로젝트 구조 생성
- `.cargo/config.toml`, `Cross.toml` 작성
- `rusqlite` 연결, 스키마 DDL 구현 (`db/schema.rs`)
- FTS5 trigram 외부콘텐츠 테이블 + 동기화 트리거
- M:N 프로젝트-문헌 매핑
- 기본 CRUD 테스트

### 마일스톤 2: 분류 체계 통합 (3주차)

- UDC Summary RDF 다운로드 + 파싱 (`rio_turtle`)
- UDC 분석합성식 기호(`+`, `:`, `/`, `-`) 데이터 모델링
- PhySH SKOS 파싱 (GitHub)
- MSC2020 CSV/SKOS 파싱
- LCC MADS/RDF 파싱 (선택)
- 분류 노드 DB 적재
- `ClassificationScheme` trait 구현
- 라벨 오버라이드 시스템 (LLM 번역 확장점 예약)
- 자동 분류 추천 엔진 프로토타입

### 마일스톤 3: PDF 파싱 + 식별자 추출 (4주차)

- `lopdf` 어댑터: XMP/카탈로그 메타데이터 추출
- `unpdf` 어댑터: 본문 텍스트 추출 (CJK/암호화/다열)
- DOI/arXiv 정규식 추출 (CrossRef 공식 패턴)
- SciPlore Xtract 휴리스틱 제목 추정
- 어댑터 트레이트 + 교체 가능 구조
- 벤치마크 (실제 학술 PDF 코퍼스)

### 마일스톤 4: CUI 기본 + 터미널 입력 (5주차)

- crossterm 초기화/복구
- 브래킷 페이스트 모드 + 드래그 앤 드롭 경로 파싱
- Kitty 키보드 프로토콜 + 폴백
- Ratatui 듀얼 패널 레이아웃
- tokio mpsc 바운드 채널 액션 디스패처
- `spawn_blocking` 격리
- 기본 이벤트 루프

### 마일스톤 5: API 통합 + 캐시 (6주차)

- 4모드 토글 구현
- CrossRef Polite Pool 클라이언트 (rustls)
- arXiv API 클라이언트
- 유형별 레이트 리미터 (2025.12 제한)
- 지수 백오프
- 로컬 디스크 캐시 (TTL 30일)

### 마일스톤 6: 인용키 + 내보내기 (7주차)

- 4모드 인용키 생성 (AuthorYear 기본)
- 커스텀 템플릿 모드 심혈 구현
- BibTeX 내보내기
- CSL JSON 내보내기

### 마일스톤 7: 파일 보관 + 설정 (8주차)

- 3모드 파일 보관 토글
- 라이브러리 폴더 관리
- 원본 휴지통 이동 (플랫폼별)
- 설정 시스템 (DB + TOML 동기화)
- 끊어진 참조 복구

### 마일스톤 8: 다플랫폼 빌드 + 테스트 (9주차)

- `cross` 설치, Docker 기반 크로스컴파일
- 4타깃 바이너리 산출 (aarch64-darwin, x86_64-linux-musl, aarch64-linux-musl, x86_64-windows-msvc)
- `cargo clippy` + `cargo test` 전 통과
- 통합 테스트 (PDF → 분류 → 검색 → 내보내기 전 경로)
- 크로스플랫폼 경로 처리 테스트

---

## 15. 라이선스 및 데이터 출처 명세

### 15.1 소프트웨어 라이선스

- Libran 본체: 저장소의 LICENSE 파일 확인 필요 (현재 존재)

### 15.2 번들 분류 데이터 라이선스

| 데이터 | 라이선스 | 출처 | 저작자 표시 요구 |
|---|---|---|---|
| UDC Summary | CC BY-SA 3.0 | Finto 미러 (UDC Consortium) | 예 (UDC Consortium 명시) |
| PhySH | CC0 | GitHub physh-org/PhySH (APS) | 아니오 (공개 도메인) |
| MSC2020 | CC BY-NC-SA | msc2020.org (MR + zbMATH) | 예 + 비영리 조건 |
| LCC | 공개 도메인 | id.loc.gov (LoC) | 아니오 |
| UDC 최상위 한국어 번역 | (자체 제작) | Libran 프로젝트 | — |

- 각 데이터의 라이선스 정보는 `classification_schemes.license` 컬럼에 저장
- 내보내기 시 라이선스 명세 포함 (UDC 사용 시 저작자 표시 의무)
- MSC2020의 NC(비영리) 조건: Libran이 순수 오픈소스 비영리인 경우 호환

### 15.3 외부 API 이용 약관

- CrossRef REST API: Polite Pool 에티켓 준수 (mailto 포함)
- arXiv API: 3초 간격 권장

---

## 16. 보안 고려사항

- **사용자 이메일**: API 통신용, DB에 저장 (평문 — 로컬 DB이므로 위험 낮으나 향후 암호화 검토)
- **API 키**: (현재 CrossRef/arXiv는 무료, 키 불필요. 향후 Semantic Scholar 등 추가 시 `app_config`에 저장, 평문)
- **PDF 보관**: 라이브러리 폴더는 사용자 홈 디렉토리 내, 권한 700 권장
- **DB 파일**: `~/.libran/libran.db`, 권한 600 권장
- **입력 검증**: 드래그 앤 드롭 경로, 사용자 입력 모두 검증 (경로 탈출, 인젝션 방지)
- **HTTP**: 모든 API 통신은 HTTPS (reqwest 기본)

---

## 17. 테스트 전략

### 17.1 단위 테스트

- `pdf/identifiers.rs`: DOI/arXiv 정규식 — 유효/무효 케이스 다수
- `citation/key_generator.rs`: 4모드별 키 생성 + 충돌 해결
- `classification/udc.rs`: 분석합성식 기호 파싱 + 검증
- `db/search.rs`: trigram FTS5 CJK 부분매칭
- `terminal/drag_drop.rs`: 경로 파싱 변칙 케이스

### 17.2 통합 테스트

- `tests/pdf_parsing.rs`: 실제 학술 PDF → 메타데이터 추출 → DB 저장 → 검색
- `tests/classification.rs`: PDF 메타데이터 → 자동 분류 추천 → 검증
- `tests/cross_platform.rs`: PathBuf 경로 처리, 라인 엔딩 정규화

### 17.3 벤치마크

- `benches/pdf_bench.rs`: lopdf + unpdf 조합의 파싱 속도 (실제 코퍼스)

---

## 부록 A: 데이터 플로우 전체도

```
[사용자 드래그 앤 드롭]
        │
        ▼
[터미널: Event::Paste(path)]
        │
        ▼
[경로 파싱: parse_dragged_path()]
        │
        ▼
[AppAction::DragDetected(path)]
        │
        ▼
[tokio::spawn → spawn_blocking]
        │
        ├─→ [lopdf: XMP/카탈로그 메타]
        ├─→ [unpdf: 본문 텍스트]
        │         │
        │         ▼
        │   [DOI/arXiv 정규식 추출]
        │         │
        │         ▼
        │   [식별자 검출?]
        │     ├─ 예 ─→ [API 모드 확인]
        │     │         ├─ IdentifierOnly ─→ [CrossRef/arXiv 조회]
        │     │         ├─ AutoFallback   ─→ [CrossRef 조회]
        │     │         ├─ ManualSearch   ─→ [대기 (사용자 명시 시)]
        │     │         └─ FullyOffline   ─→ [스킵]
        │     │
        │     └─ 아니오 ─→ [SciPlore 제목 추정]
        │                    │
        │                    ▼
        │              [API 모드 확인 (제목 검색)]
        │
        ▼
[메타데이터 통합]
        │
        ├─→ [자동 분류 추천 (UDC/PhySH/MSC)]
        ├─→ [인용키 생성]
        ├─→ [PDF 보관 (복사/참조/휴지통)]
        │
        ▼
[SQLite 저장: documents + document_classifications + documents_fts]
        │
        ▼
[CUI 갱신: 우측 패널 문헌 리스트에 추가]
        │
        ▼
[사용자: 분류 확정 / 편집 / 내보내기 (.bib / CSL JSON)]
```

## 부록 B: API 모드별 동작 매트릭스

| 상황 | IdentifierOnly | ManualSearch | AutoFallback | FullyOffline |
|---|---|---|---|---|
| DOI 검출 | CrossRef 조회 | CrossRef 조회 | CrossRef 조회 | 스킵 (저장만) |
| arXiv ID 검출 | arXiv 조회 | arXiv 조회 | arXiv 조회 | 스킵 (저장만) |
| 식별자 없음 | 메타 자체추출만 | 메타 자체추출만 | query.bibliographic 자동 검색 | 메타 자체추출만 |
| 사용자 "온라인 찾기" | 식별자 있으면 조회 | 제목 검색 수행 | 이미 자동 수행 | 거부 (오프라인) |
| 429 수신 | 백오프 후 재시도 | 백오프 후 재시도 | 백오프 후 재시도 | N/A |
| 캐시 히트 | 캐시 반환 | 캐시 반환 | 캐시 반환 | 캐시 무시 |

## 부록 C: FTS5 검색 동작 비교

| 검색어 | unicode61 (제안서) | trigram (본 설계) |
|---|---|---|
| "network" (영어) | 命中 (단어 단위) | 命中 (3-gram 매칭) |
| "ne*" (접두) | 접두 인덱스 필요 | 命中 (trigram 부분매칭) |
| "방정식" in "미분방정식해석학" | **실패** (통째로 1토큰) | **命中** (3-gram) |
| "방" (1문자) | 실패 | 실패 (3문자 미만) |
| "방정" (2문자) | 실패 | 실패 (3문자 미만) |
| LIKE '%방정식%' | 풀 스캔 | **인덱스 가속** |

---

*본 문서는 구현의 단일 진실 원(single source of truth)이다. 구현 중 발생하는 변경은 본 문서에 먼저 반영한다.*
