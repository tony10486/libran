# Libran

Libran은 Rust 기반 오프라인-first 서지 관리 소프트웨어입니다. 모든 메타데이터 추출은 로컬 PDF 파싱으로 수행하며, 외부 API는 옵션으로 동작합니다. 터미널 듀얼 패널 CUI 인터페이스를 제공합니다.

# Features

## 문헌 분류

### UDC
Libran은 국제십진분류법(Universal Decimal Classification, UDC)를 참고하여 문헌을 분류합니다. UDC는 문헌을 분석합성식으로 분류하여, 다학제적 연구를 편리하게 찾을 수 있다는 특징이 있습니다. 분석합성식이란 기존의 분류 체계가 대응할 수 없는 자세한 분류가 필요할 때에도, 기존의 체계를 유지할 수 있도록 계층식으로 분석함과 동시에, 보조 기호 등을 통해 문헌의 정확한 정보를 코드로 분류할 수 있는 체계를 말합니다.

#### Example
UDC 분류가 실제 논문에 적용된 사례를 보며 이해해 봅시다. [_Nonisothermal diffuse interface model for electrical breakdown channel propagation_](https://doi.org/10.33048/SIBJIM.2022.25.103)의 UDC 코드는 `51-72:517.9:538.91`로, `51-72`는 수학에 해당하는 `51`번 분야에서, `-7`이 수학적 모델링과 타 학문으로의 응용이 있음을 뜻하며, `-72`는 그 중에서 물리학 및 역학 분야에 적용되는 수학적 모델링을 의미합니다. 그리고 `:`를 사용하여 융합된 학문에 대한 정보를 밝힙니다. `517`은 수학 중에서 해석학에 해당하며, `.9`는 해석학 중 미분방정식과 적분방정식의 범주입니다. 따라서 `517.9`는 미분방정식 또는 적분방정식을 해석학적 방법론으로 연구함을 의미합니다. 이 연구가 적용된 분과는 그 다음 `:`에 오는데, 물리학에 해당하는 `53`에서, `538.9`가 응집물질물리학을 의미하고, `1`이 추가적으로 응집물질물리학 중에서도 물질의 구조 및 상전이에 대한 연구를 의미합니다. 따라서 `51-72:517.9:538.91`로 하여금 해당 논문이 "응집물질물리학에서 일어나는 미시적인 물리 현상을 해석하기 위해 해석학적 방법을 적용한 미분방정식의 연구를 기반으로 설계한 수학적 모델링을 다루는 논문"이라고 알 수 있습니다.

이처럼 문헌 관리와, 연구자가 원하는 논문을 직관적으로 찾을 수 있다는 장점을 가지고 있으며, 기존의 저널이나 도서관 등에서도 널리 사용되고 있는 체계가 UDC라는 점에서 해당 체계를 선택하였습니다.

### 기타 분류법
각 분야에서도 독자적인 분류법을 사용하고 있기도 합니다. 수학의 경우 미국수학회가 제시한 분류 체계인 수학분야분류법(Mathematics Subject Classification, MSC)가 사용되고 있으며, 물리와 화학은 각각 PhySH, CAS의 섹션 코드를 사용하고 있습니다. Libran에서는 각 분류 체계를 UDC에 대응하되, 기존의 체계를 통한 검색 또한 지원합니다.

### 분류 데이터 번들
Libran은 UDC(87개 노드), PhySH(45개 노드), MSC(63개 노드)의 분류 데이터를 한국어 번역과 함께 바이너리에 임베드하여 제공합니다. 프로그램 시작 시 자동으로 SQLite 데이터베이스에 등록되며, 별도의 외부 데이터 다운로드가 필요하지 않습니다.

### 커스텀 분류 체계 가져오기
`:import-classification <CSV 파일 경로>` 명령으로 사용자 정의 분류 체계를 CSV 파일에서 가져올 수 있습니다. CSV 형식은 다음과 같습니다:

```
notation,pref_label,broader_notation,alt_labels,notes
1,Root,,,root note
1.1,Child A,1,alt1;alt2,child note
1.2,Child B,1,,
```

- `notation`: 분류 기호 (필수)
- `pref_label`: 대표 라벨 (필수)
- `broader_notation`: 상위 분류 기호 (선택, 비워두면 최상위 노드)
- `alt_labels`: 대체 라벨, 세미콜론(`;`) 구분 (선택)
- `notes`: 범위 주석 (선택)

스킴 코드는 파일명에서 파생됩니다 (예: `my-scheme.csv` → 코드 `my-scheme`). 동일한 기호가 중복되면 첫 번째 항목만 저장됩니다. 계층 구조를 유지하려면 상위 노드를 하위 노드보다 먼저 배치하세요.

### 자동 분류 추천
PDF에서 추출된 메타데이터(제목, 저널명, 키워드)를 기반으로 UDC 코드를 자동 추천합니다. 추천은 제안만 하고, 최종 분류는 항상 사용자가 확정합니다.

## 문헌 검색

SQLite FTS5 trigram 토크나이저를 사용한 전문 검색을 제공합니다. CJK(한국어, 중국어, 일본어) 텍스트의 부분 매칭을 지원하며, 제목, 저자, 저널, 초록, 키워드 필드를 통합적으로 검색합니다. `/` 키를 눌러 검색 모드로 진입하여 검색어를 입력할 수 있습니다.

### 전문 색인 (본문 검색)
PDF 본문 텍스트를 추출하여 FTS5 trigram 인덱스에 저장합니다. 메타데이터 검색과 본문 검색을 토글할 수 있어, 문헌의 메타데이터뿐만 아니라 본문 내용으로도 검색할 수 있습니다. 약 95kB/문헌의 추가 저장 공간이 필요합니다.

## 자동 서지 정보 가져오기

### PDF 파싱 파이프라인
PDF 파일을 터미널로 드래그 앤 드롭하거나 `a` 키로 경로를 입력하면 자동으로 메타데이터를 추출합니다.

- **lopdf**: PDF 내부 카탈로그에서 Title, Author, Subject 메타데이터 추출
- **unpdf**: 본문 텍스트 추출, CJK/RTL 폰트 처리, 암호화 PDF 자동 복호화
- **DOI/arXiv ID 정규식 추출**: CrossRef 공식 패턴 기반 DOI 추출, arXiv 신형/구형 ID 동시 감지. 전체 텍스트에서 검색하며, 실패 시 파일명에서도 추출
- **제목 추정 휴리스틱**: 첫 페이지에서 Abstract 마커 전의 텍스트를 분석하여 제목 후보 식별. 실패 시 전체 텍스트에서 역순 탐색. 소문자 시작 줄, 문장형 줄, 섹션 번호 줄을 제외하여 정확도 향상

### 외부 API 통합 (4모드 토글)
`o` 키로 API 모드를 토글할 수 있습니다.

| 모드 | 설명 |
|---|---|
| 오프라인 | 외부 API 전면 배제. 모든 메타는 PDF 자체 추출만 |
| 식별자만 | DOI/arXiv ID가 검출된 경우만 CrossRef/arXiv 직접 조회 |
| 자동 폴백 | 식별자 없으면 제목으로 CrossRef 검색 폴백 |

- **CrossRef**: Polite Pool 클라이언트, 유형별 레이트 리미트 (단일 DOI 10/s, 목록 검색 3/s), 지수 백오프
- **arXiv**: Atom XML 응답 파싱하여 제목, 저자, 초록, 출판 연도 추출
- **OpenAlex**: 전방 인용(이 문헌을 인용한 문헌) 조회. `F` 키로 실행하며, 조회된 인용 문헌은 자동으로 라이브러리에 저장되고 인용 관계가 생성됩니다
- **Semantic Scholar / OpenAlex 메트릭스**: 인용 수, 영향도 지표 조회
- **API 캐시**: 응답을 `api_cache` 테이블에 TTL 30일로 저장, 동일 식별자 재호출 방지
- PDF 추출 후 자동으로 API 조회를 시도하여 빈 필드(저자, 저널, 연도, 초록 등)를 보강합니다

### 중복 문헌 감지
strsim 기반 퍼지 매칭으로 메타데이터가 유사한 중복 문헌을 감지합니다. 제목(가중치 3.0, Jaro-Winkler), 저자(가중치 2.5), 연도(가중치 1.0, 정확 매치)의 가중 평균으로 유사도를 계산하며, 임계값 0.75 이상이면 중복으로 판정합니다. 대량 가져오기 시 자동으로 경고를 표시합니다. 기존의 SHA-256 파일 해시 및 DOI 중복 감지와 함께 3단계 중복 감지를 제공합니다.

## 항목 유형

8가지 항목 유형을 지원하여 문헌의 종류에 따른 정확한 메타데이터 관리와 내보내기를 제공합니다.

| 유형 | 설명 | CSL 매핑 | BibTeX 매핑 |
|---|---|---|---|
| `article` | 학술 논문 | `article-journal` | `@article` |
| `book` | 도서 | `book` | `@book` |
| `thesis` | 학위 논문 | `thesis` | `@phdthesis` |
| `conference` | 회의 논문 | `paper-conference` | `@inproceedings` |
| `dataset` | 데이터셋 | `dataset` | `@misc` |
| `webpage` | 웹 페이지 | `webpage` | `@misc` |
| `patent` | 특허 | `patent` | `@misc` |
| `misc` | 기타 | `document` | `@misc` |

PDF 가져오기 시 메타데이터를 기반으로 항목 유형을 자동 추론하며(저널명 있음 → article, ISBN 있음 → book 등), 사용자가 `e` 키 편집 모드에서 "유형" 필드로 직접 변경할 수 있습니다. 모든 내보내기 형식이 항목 유형에 따라 타입 인식 출력을 생성합니다.

## 구조화된 저자

저자를 단일 텍스트 필드가 아닌 구조화된 레코드로 관리합니다. 각 저자는 다음 필드를 가집니다:

- **creator_type**: `author`, `editor`, `translator`, `contributor` 중 하나
- **family**: 성
- **given**: 이름
- **suffix**: 접미사 (Jr., III 등)
- **particles**: 이름 입자 (van, de 등)
- **literal**: 리터럴 형식 (CJK 이름 등 분할이 부정확한 경우)
- **locale**: 언어/지역 코드 (`ko`, `ja`, `zh`, `en` 등)
- **order_index**: 저자 순서

CJK 저자 이름은 Unicode 범위 감지로 locale을 자동 추론합니다 (Hangul → `ko`, Kana → `ja`, Han-only → `zh`). 기존 `authors` 텍스트 필드와 듀얼 라이트로 동작하여 하위 호환성을 유지합니다.

### CJK 인용 렌더링
한국어, 중국어, 일본어 저자 이름을 올바르게 렌더링합니다. CJK 이름을 감지하여 family-first 순서를 적용하고, 이니셜을 생략하며, literal 형식을 사용합니다. per-creator `locale` 필드를 통해 각 저자의 언어에 맞는 정확한 인용 형식을 생성합니다. 예: "김철수" → "김철수" (family-first, 이니셜 없음), "Smith, John" → "Smith, J." (Western 형식 유지).

## 프로젝트 별 관리

문헌을 프로젝트별로 그룹화할 수 있습니다. `n` 키로 새 프로젝트를 생성하고, 좌측 패널에서 프로젝트를 선택하여 해당 프로젝트의 문헌만 필터링할 수 있습니다. 프로젝트-문헌은 M:N 관계로 하나의 문헌이 여러 프로젝트에 속할 수 있습니다.

## 문헌 수정

`e` 키를 눌러 문헌의 메타데이터를 편집할 수 있습니다. 9개 필드(제목, 저자, 저널, 연도, DOI, arXiv ID, 초록, 키워드, 유형)를 `Tab` 키로 순회하며 편집할 수 있습니다. `d` 키로 문헌을 삭제할 수 있습니다.

## 노트 (다중 메모)

문헌별로 여러 개의 마크다운 노트를 작성할 수 있습니다. 노트는 SQLite 데이터베이스에 저장되어 검색 및 쿼리가 가능합니다.

- `n` 키: 현재 문헌에 새 노트 작성 (인라인 입력)
- `:note` 명령: `$EDITOR` 환경변수로 지정된 외부 에디터(vi가 기본값)로 노트 작성
- 노트는 `updated_at` 기준으로 정렬되며, 상세 보기에서 최신 노트와 노트 수가 표시됩니다

## 태그 및 즐겨찾기

### 색상 태그
태그에 색상을 지정할 수 있습니다. `:tag-color <태그> <hex>` 명령으로 색상을 지정하고, `:tag-color <태그>` (색상 인수 없이)로 색상을 제거합니다. 색상은 태그 이름별로 전역 적용되며, 우측 패널 상세 보기에서 색상이 렌더링됩니다.

### 즐겨찾기
`*` 키로 즐겨찾기 필터를 토글할 수 있습니다. 평점(rating)이 5인 문헌이 즐겨찾기로 표시됩니다.

## 읽기 큐 (TBR)

읽기 대기열(To Be Read)을 관리할 수 있습니다.

| 키 | 기능 |
|---|---|
| `Q` | 현재 문헌을 읽기 큐에 추가 |
| `R` | 현재 문헌을 읽기 큐에서 제거 |
| `Y` | 읽기 큐 보기 토글 |
| `>` | 읽기 진행률 +10% |
| `<` | 읽기 진행률 -10% |

읽기 큐 보기 모드에서 `j`/`k`로 큐 내 탐색, `Enter`로 문헌 열기, `Esc`로 큐 보기 종료할 수 있습니다. 큐 순서는 변경 가능합니다.

## 스마트 컬렉션

저장된 검색에 구조화된 조건을 지정할 수 있습니다. 다음 조건으로 필터링할 수 있습니다:

| 조건 필드 | 연산자 | 설명 |
|---|---|---|
| `tag` | equals | 특정 태그가 있는 문헌 |
| `year` | equals | 특정 연도의 문헌 |
| `year_range` | between | 연도 범위 (예: `"2020-2024"`) |
| `reading_status` | equals | 읽기 상태 (unread/reading/read) |
| `rating` | equals/gte/lte | 평점 조건 |
| `author` | equals/contains | 저자명 조건 |
| `journal` | equals/contains | 저널명 조건 |
| `classification` | equals | 분류 기호 조건 |

조건 간의 결합 모드로 `all`(AND)과 `any`(OR)를 지원합니다. 조건은 `filters_json` 컬럼에 JSON으로 저장됩니다.

## 전방 인용 추적

OpenAlex API를 통해 특정 문헌을 인용한 문헌(전방 인용)을 조회합니다. `F` 키로 실행하며, 조회된 인용 문헌은 자동으로 라이브러리에 문헌으로 저장되고, 원본 문헌과의 인용 관계(citation_relations)가 생성됩니다. 인용 수는 상태 표시줄에 표시됩니다.

## 문헌 내보내기

`x` 키로 내보내기 대화상자를 엽니다. 인용 스타일(15종), 내보내기 형식(16종 + 커스텀), 언어, 표시 모드를 선택할 수 있으며, `Enter`로 클립보드에 인용 텍스트를 복사하고 `e`로 파일로 내보낼 수 있습니다.

### 내보내기 형식 (16종 + 커스텀)

| 형식 | 설명 |
|---|---|
| **BibTeX (.bib)** | BibTeX 엔트리 형식. 인용키, 저자, 제목, 저널, 연도, DOI, arXiv, 키워드, 파일 경로 포함 |
| **CSL JSON** | Citation Style Language 호환 JSON. Typst, LaTeX, pandoc, Zotero 등 외부 도구에서 직접 소비 가능 |
| **RIS** | 레퍼런스 매니저 범용 포맷 (TY/AU/TI/PY/JO/DO/ER 태그) |
| **CSV** | 표 형태 내보내기 (RFC 4180 준수) |
| **MODS XML** | 미국 의회 도서관 MODS 3.7 스키마 |
| **Endnote XML** | EndNote XML 형식 |
| **Bibliontology RDF** | RDF/OWL 서지 온톨로지 |
| **Bookmarks** | 브라우저 북마크 형식 |
| **CFF** | Citation File Format (GitHub용) |
| **CFF References** | CFF 참조 형식 |
| **COinS** | ContextObjects in Spans (웹 페이지 임베딩용) |
| **Refer/BibIX** | Refer/BibIX 형식 |
| **RefWorks Tagged** | RefWorks 태그 형식 |
| **Evernote ENEX** | Evernote 내보내기 형식 |
| **TEI** | Text Encoding Initiative XML |
| **Wikidata QuickStatements** | Wikidata 일괄 편집 형식 |
| **커스텀** | 사용자 정의 템플릿 (설정 파일에서 등록) |

### 인용 스타일 (15종)

| 스타일 | 설명 |
|---|---|
| **APA 7th** | American Psychological Association 7th Edition |
| **ACS** | American Chemical Society |
| **AMA** | American Medical Association |
| **APSA** | American Political Science Association |
| **ASA** | American Sociological Association |
| **Chicago 18th** | Chicago Manual of Style 18th Edition |
| **Cite Them Right Harvard** | Harvard 스타일 (Cite Them Right) |
| **Elsevier Harvard** | Elsevier Harvard 스타일 |
| **IEEE** | Institute of Electrical and Electronics Engineers |
| **MHRA** | Modern Humanities Research Association |
| **MLA 9th** | Modern Language Association 9th Edition |
| **Nature** | Nature 저널 스타일 |
| **NLM/Vancouver** | National Library of Medicine / Vancouver 스타일 |

각 스타일은 참고문헌 목록과 인용 텍스트(1/2/3+ 저자 et al. 규칙)를 지원합니다.

### 사용자 데이터 포함 내보내기

CSL JSON, BibTeX, RIS, CSV 내보내기에 사용자가 생성한 데이터가 포함됩니다:

- **노트**: 문헌에 작성한 모든 노트
- **태그**: 문헌에 지정된 태그
- **분류**: UDC/PhySH/MSC 분류 정보
- **프로젝트**: 소속된 프로젝트 목록
- **읽기 상태**: 읽음/읽는 중/안 읽음
- **커스텀 필드**: 사용자 정의 필드

또한 전체 라이브러리를 JSON으로 덤프하는 기능을 제공하여, 모든 문헌과 사용자 데이터를 한 번에 내보낼 수 있습니다.

### 인용키 생성
4가지 모드를 지원합니다:

| 모드 | 형식 | 예시 |
|---|---|---|
| 성+연도 | `{성}{연도}` | `Smith2024` |
| 성+연도+제목 | `{성}{연도}{제목첫단어}` | `Smith2024Nonisothermal` |
| 성+연도+해시 | `{성}{연도}{단축해시}` | `Smith2024a3f2b1` |
| 커스텀 템플릿 | 사용자 정의 템플릿 | `{author}_{year2}_{titleword}` |

다국어 저자(한글, 중문, 일문)를 원본 그대로 보존하며, 중복 시 a, b, c... 접미사로 충돌을 해결합니다.

## 파일 보관

PDF 파일을 라이브러리 폴더(`~/.libran/library/`)로 복사하여 관리합니다. SHA-256 해시로 중복 파일을 감지하며, 파일명은 `{인용키}.pdf` 형식으로 저장됩니다.

### 다중 첨부 파일
문헌에 PDF 외의 파일을 첨부할 수 있습니다. EPUB, HTML, 보충 자료, 데이터셋, 슬라이드 등 다양한 파일 형식을 지원합니다. 첨부 파일은 `document_attachments` 테이블로 관리되며, 파일명은 `{인용키}_att{n}.{확장자}` 형식으로 저장됩니다. 기존 `documents.file_path`는 주 첨부 파일(Primary PDF)로 유지되어 하위 호환성이 보장됩니다.

## 백업 및 복구

`:backup <경로>` 명령으로 데이터베이스를 백업할 수 있습니다. SQLite의 `VACUUM INTO` 명령을 사용하여 WAL 모드에서도 안전하게 백업합니다 (읽기 잠금만 사용, 단일 파일로 컴팩트하게 생성).

`:restore <경로>` 명령으로 백업 파일에서 데이터베이스를 복원할 수 있습니다. 복원 후 프로그램 재시작이 필요합니다.

```
:backup ~/.libran/backup_20260627.db
:restore ~/.libran/backup_20260627.db
```

## CUI 인터페이스

완전 다크 모드의 터미널 인터페이스를 제공합니다. 테두리 없이 색상 차이로 섹션을 구분합니다.

### 레이아웃
- **헤더**: Libran 로고, 활성 프로젝트명, 문헌 수, API 모드, 온라인/오프라인 상태
- **좌측 패널 (32%)**: 프로젝트 리스트 + UDC 분류 트리 (확장/축소 가능) + PhySH (해당 분류 문헌이 있을 때만 표시)
- **우측 패널 (68%)**: 문헌 리스트 (제목, 저자, 연도, DOI, 인용키 표시)
- **상세 모드**: `Enter` 키로 3패널 상세 보기 전환 (제목, 저자, 저널, 연도, DOI, arXiv, 인용키, 파일 경로, 초록, 노트, 태그, 첨부 파일)
- **읽기 큐 보기**: `Y` 키로 읽기 큐 전용 보기 전환
- **상태 표시줄**: 현재 상태, API 모드, 문헌 수, 단축키 힌트

### 단축키

| 키 | 기능 |
|---|---|
| `Tab` | 패널 간 포커스 이동 |
| `j` / `k` | 위/아래 탐색 |
| `Enter` | 문헌 상세 보기 / 닫기 |
| `/` | 검색 모드 진입 |
| `a` | 파일 경로 입력 추가 |
| `Space` | 문헌 다중 선택 토글 |
| `e` | 문헌 메타데이터 편집 (9개 필드) |
| `d` | 문헌 삭제 |
| `x` | 내보내기 대화상자 (인용 복사 + 파일 내보내기) |
| `n` | 새 프로젝트 생성 / 새 노트 작성 (상세 모드) |
| `o` | API 모드 토글 |
| `?` | 도움말 |
| `q` / `Esc` | 종료 |
| `*` | 즐겨찾기 필터 토글 (rating=5) |
| `Q` | 읽기 큐에 추가 |
| `R` | 읽기 큐에서 제거 |
| `Y` | 읽기 큐 보기 토글 |
| `>` | 읽기 진행률 +10% |
| `<` | 읽기 진행률 -10% |
| `F` | 전방 인용 조회 (OpenAlex) |
| `u` | 읽음 상태 토글 |
| `b` | 북마크/TOC 보기 |
| `p` | 외부 PDF 뷰어로 열기 |
| `:` | 명령 모드 진입 |

### 명령 모드

`:` 키를 눌러 명령 모드로 진입할 수 있습니다. vim 스타일의 명령 입력을 지원합니다.

| 명령 | 기능 |
|---|---|
| `:backup <경로>` | 데이터베이스 백업 (VACUUM INTO, WAL 안전) |
| `:restore <경로>` | 데이터베이스 복원 (재시작 필요) |
| `:tag-color <태그> <hex>` | 태그 색상 지정 (예: `:tag-color important #ff0000`) |
| `:tag-color <태그>` | 태그 색상 제거 |
| `:import-classification <경로>` | 커스텀 분류 체계 CSV 가져오기 |
| `:note` | `$EDITOR`로 노트 작성 |

### 드래그 앤 드롭
터미널로 PDF 파일을 드래그하여 추가할 수 있습니다. iTerm2, Kitty, WezTerm 등 브래킷 페이스트 모드를 지원하는 터미널에서 동작합니다. `file://` URL, URL 인코딩, 따옴표, 공백 이스케이프를 자동으로 처리합니다.

## 데이터베이스

SQLite(rusqlite bundled)를 사용합니다. 데이터베이스 파일은 `~/.libran/libran.db`에 위치합니다.

- **WAL 모드**: 동시성 향상을 위한 Write-Ahead Logging
- **FTS5 trigram**: CJK 부분 매칭을 지원하는 전문 검색 (메타데이터 + 본문)
- **외부 콘텐츠 테이블**: FTS 인덱스만 저장하여 용량 절약
- **트리거**: INSERT/UPDATE/DELETE 시 자동 FTS 동기화
- **마이그레이션**: M1-M17 (17개 버전 마이그레이션, 자동 업그레이드)

### 주요 테이블

| 테이블 | 설명 |
|---|---|
| `documents` | 문헌 메타데이터 (27+ 컬럼, item_type, queue_position 등) |
| `documents_fts` | 메타데이터 FTS5 전문 검색 인덱스 |
| `documents_body` | PDF 본문 텍스트 |
| `documents_body_fts` | 본문 FTS5 전문 검색 인덱스 |
| `creators` | 구조화된 저자 (creator_type, family, given, locale 등) |
| `document_attachments` | 다중 첨부 파일 (PDF 외 EPUB, HTML 등) |
| `document_notes` | 다중 노트 (마크다운, note_type) |
| `tags` | 태그 (color 컬럼 포함) |
| `citation_relations` | 인용 관계 (전방/후방 인용) |
| `saved_searches` | 저장된 검색 (fts_query + filters_json) |
| `classification_schemes` | 분류 체계 레지스트리 |
| `classification_nodes` | 분류 노드 (계층 구조) |
| `projects` | 프로젝트 |
| `api_cache` | API 응답 캐시 (TTL 30일) |
| `app_config` | 애플리케이션 설정 (JSON) |

### 설정 영속화
애플리케이션 설정(API 모드, 라이브러리 경로, 인용키 모드, 활성 스킴 등)은 `app_config` 테이블에 JSON으로 저장됩니다.

## 로깅

`~/.libran/libran.log`에 디버그 로그를 기록합니다. 터미널 이벤트, PDF 처리, API 조회, 경로 파싱 결과 등이 기록되어 문제 진단에 활용할 수 있습니다.

## Libran 확장하기

Libran은 여러 확장 지점을 제공합니다. 각 확장 지점은 설정 파일(`~/.libran/config.toml`) 또는 Rust 트레이트 구현을 통해 사용자 정의할 수 있습니다.

### 커스텀 분류 체계

Libran은 `ClassificationScheme` 트레이트를 통해 분류 체계를 확장할 수 있습니다. 트레이트는 `src/classification/scheme.rs`에 정의되어 있으며, 다음 메서드를 구현해야 합니다:

- `code()`: 분류 체계 코드 (예: `"udc"`, `"my-scheme"`)
- `name()`: 분류 체계 이름
- `version()`, `license()`, `source_url()`: 메타데이터
- `is_primary()`: 주 분류 체계 여부
- `nodes()`: 분류 노드 목록 (`ClassificationNode` 배열)
- `validate_notation()`: 기호 유효성 검사

커스텀 분류 체계 구현 예시는 `src/classification/custom.rs`의 `CustomScheme`을 참조하세요.

#### CSV로 분류 체계 가져오기

Rust 코드를 작성하지 않고도 CSV 파일로 분류 체계를 가져올 수 있습니다. `:import-classification <CSV 파일 경로>` 명령을 사용하세요. CSV 형식은 위 "커스텀 분류 체계 가져오기" 섹션을 참조하세요.

### 커스텀 내보내기 형식

`~/.libran/config.toml`에 커스텀 내보내기 형식을 정의할 수 있습니다. 템플릿 기반 문자열 치환으로 원하는 형식으로 문헌을 내보낼 수 있습니다.

```toml
[[custom_export_formats]]
name = "plain_text"
file_extension = "txt"
template = "{title} - {authors} ({year}). {doi}"

[[custom_export_formats]]
name = "markdown_ref"
file_extension = "md"
template = "- [{title}]({doi}) — {authors}, {year}"
```

**템플릿 플레이스홀더**:

| 플레이스홀더 | 치환 값 | 비고 |
|---|---|---|
| `{title}` | 문헌 제목 | |
| `{authors}` | 저자 문자열 | 세미콜론(`;`) 구분 |
| `{year}` | 출판 연도 | 값이 없으면 빈 문자열 |
| `{doi}` | DOI | 값이 없으면 빈 문자열 |
| `{journal}` | 저널명 | 값이 없으면 빈 문자열 |
| `{abstract}` | 초록 | 값이 없으면 빈 문자열 |

여러 문헌을 내보낼 때 각 문헌의 치환 결과가 줄바꿈으로 구분되어 출력됩니다.

### 커스텀 인용키 템플릿

`citation_key_mode = "custom"`으로 설정하고 `citation_key_template`에 템플릿을 지정하여 인용키 생성 형식을 사용자 정의할 수 있습니다.

```toml
citation_key_mode = "custom"
citation_key_template = "{author}_{year2}_{titleword}"
```

사용 가능한 플레이스홀더: `{author}` (첫 저자 성), `{year2}` (연도 뒤 2자리), `{titleword}` (제목 첫 단어). 다국어 저자(한글, 중문, 일문)는 원본 그대로 보존되며, 중복 시 a, b, c... 접미사로 충돌을 해결합니다.

### 설정 키 (app_config)

`~/.libran/config.toml`에서 다음 설정을 구성할 수 있습니다:

| 키 | 설명 | 기본값 |
|---|---|---|
| `api_mode` | API 조회 모드 (`IdentifierOnly` / `AutoFallback` / `ManualSearch` / `FullyOffline`) | `IdentifierOnly` |
| `user_email` | CrossRef polite 요청 이메일 | (없음) |
| `file_storage_policy` | 파일 저장 정책 (`CopyToLibrary` / `ReferenceOnly` / `CopyAndTrash`) | `CopyToLibrary` |
| `library_path` | PDF 라이브러리 경로 | `~/.libran/library` |
| `citation_key_mode` | 인용키 생성 모드 | `AuthorYear` |
| `citation_key_template` | 커스텀 인용키 템플릿 | (없음) |
| `primary_scheme` | 주 분류 체계 | `udc` |
| `enabled_schemes` | 활성화된 분류 체계 목록 | `["udc", "physh", "msc"]` |
| `label_language` | 분류 라벨 언어 | `en` |
| `db_path` | 데이터베이스 파일 경로 | `~/.libran/libran.db` |
| `viewer_command` | 외부 PDF 뷰어 명령 | (시스템 기본값) |
| `glyph_set` | 읽음 상태 마커 글리프 (`circles` / `ballot`) | `circles` |
| `theme` | UI 테마 색상 | (기본 테마) |
| `custom_export_formats` | 커스텀 내보내기 형식 목록 | `[]` |
