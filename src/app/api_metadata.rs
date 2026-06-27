use crate::api;
use crate::db::documents;
use crate::pdf;

use super::AppState;
use super::action::AppAction;

pub(crate) fn try_api_lookup(
    tx: tokio::sync::mpsc::Sender<AppAction>,
    mode: api::ApiMode,
    doc: &documents::Document,
) {
    let doi = doc.doi.clone();
    let arxiv_id = doc.arxiv_id.clone();
    let doc_id = doc.id.unwrap_or(0);
    let title = doc.title.clone();

    tokio::spawn(async move {
        if let Some(doi) = doi {
            match api::crossref::create_polite_http_client(None) {
                Ok(client) => match api::crossref::fetch_by_doi(&client, &doi).await {
                    Ok(body) => {
                        if let Some(meta) = parse_crossref_response(&body) {
                            let _ = tx.send(AppAction::ApiLookupSuccess(meta, doc_id)).await;
                        } else {
                            let _ = tx
                                .send(AppAction::ApiLookupSkipped(
                                    "CrossRef 응답 파싱 실패".to_string(),
                                ))
                                .await;
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(AppAction::ApiLookupFailed(e.to_string())).await;
                    }
                },
                Err(e) => {
                    let _ = tx.send(AppAction::ApiLookupFailed(e.to_string())).await;
                }
            }
        } else if let Some(arxiv) = arxiv_id {
            match api::arxiv::create_client() {
                Ok(client) => match api::arxiv::fetch_by_arxiv_id(&client, &arxiv).await {
                    Ok(body) => {
                        if let Some(meta) = parse_arxiv_response(&body) {
                            let _ = tx.send(AppAction::ApiLookupSuccess(meta, doc_id)).await;
                        } else {
                            let _ = tx
                                .send(AppAction::ApiLookupSkipped(
                                    "arXiv 응답 파싱 실패".to_string(),
                                ))
                                .await;
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(AppAction::ApiLookupFailed(e.to_string())).await;
                    }
                },
                Err(e) => {
                    let _ = tx.send(AppAction::ApiLookupFailed(e.to_string())).await;
                }
            }
        } else if mode == api::ApiMode::AutoFallback {
            match api::crossref::create_polite_http_client(None) {
                Ok(client) => match api::crossref::search_by_title(&client, &title).await {
                    Ok(body) => {
                        if let Some(meta) = parse_crossref_search_response(&body) {
                            let _ = tx.send(AppAction::ApiLookupSuccess(meta, doc_id)).await;
                        } else {
                            let _ = tx
                                .send(AppAction::ApiLookupSkipped(
                                    "제목 검색 결과 없음".to_string(),
                                ))
                                .await;
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(AppAction::ApiLookupFailed(e.to_string())).await;
                    }
                },
                Err(e) => {
                    let _ = tx.send(AppAction::ApiLookupFailed(e.to_string())).await;
                }
            }
        } else {
            let _ = tx
                .send(AppAction::ApiLookupSkipped("식별자 없음".to_string()))
                .await;
        }
    });
}

pub(crate) fn parse_crossref_response(body: &str) -> Option<pdf::RawMetadata> {
    let json: serde_json::Value = serde_json::from_str(body).ok()?;
    let message = json.get("message")?;
    let title = message
        .get("title")
        .and_then(|t| t.as_array())
        .and_then(|a| a.first())
        .and_then(|t| t.as_str());
    let authors = message
        .get("author")
        .and_then(|a| a.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|a| {
                    let family = a.get("family").and_then(|f| f.as_str()).unwrap_or("");
                    let given = a.get("given").and_then(|g| g.as_str()).unwrap_or("");
                    if !family.is_empty() {
                        if given.is_empty() {
                            Some(family.to_string())
                        } else {
                            Some(format!("{}, {}", family, given))
                        }
                    } else {
                        a.get("name")
                            .and_then(|n| n.as_str())
                            .map(|s| s.to_string())
                    }
                })
                .collect::<Vec<String>>()
        })
        .unwrap_or_default();

    let journal = message
        .get("container-title")
        .and_then(|t| t.as_array())
        .and_then(|a| a.first())
        .and_then(|t| t.as_str());
    let year = message
        .get("published-print")
        .or_else(|| message.get("published-online"))
        .or_else(|| message.get("issued"))
        .and_then(|d| d.get("date-parts"))
        .and_then(|d| d.as_array())
        .and_then(|a| a.first())
        .and_then(|a| a.as_array())
        .and_then(|a| a.first())
        .and_then(|y| y.as_i64());
    let doi = message.get("DOI").and_then(|d| d.as_str());
    let abstract_text = message
        .get("abstract")
        .and_then(|a| a.as_str())
        .map(strip_jats_tags);

    Some(pdf::RawMetadata {
        title: title.map(|s| s.to_string()),
        authors,
        journal: journal.map(|s| s.to_string()),
        pub_year: year,
        doi: doi.map(|s| s.to_string()),
        arxiv_id: None,
        abstract_text,
        keywords: Vec::new(),
        source: pdf::MetadataSource::Crossref,
        body_text: None,
    })
}

pub(crate) fn strip_jats_tags(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut in_tag = false;
    for c in s.chars() {
        if c == '<' {
            in_tag = true;
        } else if c == '>' {
            in_tag = false;
        } else if !in_tag {
            result.push(c);
        }
    }
    result.trim().to_string()
}

pub(crate) fn parse_crossref_search_response(body: &str) -> Option<pdf::RawMetadata> {
    let json: serde_json::Value = serde_json::from_str(body).ok()?;
    let items = json
        .get("message")
        .and_then(|m| m.get("items"))
        .and_then(|i| i.as_array())?;
    let first = items.first()?;
    let title = first
        .get("title")
        .and_then(|t| t.as_array())
        .and_then(|a| a.first())
        .and_then(|t| t.as_str());
    let authors = first
        .get("author")
        .and_then(|a| a.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|a| {
                    let family = a.get("family").and_then(|f| f.as_str()).unwrap_or("");
                    let given = a.get("given").and_then(|g| g.as_str()).unwrap_or("");
                    if !family.is_empty() {
                        if given.is_empty() {
                            Some(family.to_string())
                        } else {
                            Some(format!("{}, {}", family, given))
                        }
                    } else {
                        None
                    }
                })
                .collect::<Vec<String>>()
        })
        .unwrap_or_default();
    let journal = first
        .get("container-title")
        .and_then(|t| t.as_array())
        .and_then(|a| a.first())
        .and_then(|t| t.as_str());
    let year = first
        .get("published-print")
        .or_else(|| first.get("issued"))
        .and_then(|d| d.get("date-parts"))
        .and_then(|d| d.as_array())
        .and_then(|a| a.first())
        .and_then(|a| a.as_array())
        .and_then(|a| a.first())
        .and_then(|y| y.as_i64());
    let doi = first.get("DOI").and_then(|d| d.as_str());

    Some(pdf::RawMetadata {
        title: title.map(|s| s.to_string()),
        authors,
        journal: journal.map(|s| s.to_string()),
        pub_year: year,
        doi: doi.map(|s| s.to_string()),
        arxiv_id: None,
        abstract_text: None,
        keywords: Vec::new(),
        source: pdf::MetadataSource::Crossref,
        body_text: None,
    })
}

pub(crate) fn parse_arxiv_response(body: &str) -> Option<pdf::RawMetadata> {
    let mut buf = Vec::new();

    let mut title: Option<String> = None;
    let mut authors: Vec<String> = Vec::new();
    let mut abstract_text: Option<String> = None;
    let mut year: Option<i64> = None;

    use quick_xml::events::Event;
    let mut reader = quick_xml::Reader::from_str(body);
    let mut in_title = false;
    let mut in_summary = false;
    let mut in_published = false;
    let mut in_name = false;
    let mut current_name = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match name.as_str() {
                    "title" => in_title = true,
                    "summary" => in_summary = true,
                    "published" => in_published = true,
                    "name" => {
                        in_name = true;
                        current_name.clear();
                    }
                    _ => {}
                }
            }
            Ok(Event::End(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match name.as_str() {
                    "title" => in_title = false,
                    "summary" => in_summary = false,
                    "published" => in_published = false,
                    "name" => {
                        in_name = false;
                        if !current_name.is_empty() {
                            authors.push(current_name.clone());
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(e)) => {
                let text = e.unescape().ok().map(|s| s.to_string()).unwrap_or_default();
                if in_title && title.is_none() {
                    title = Some(text.clone());
                }
                if in_summary {
                    abstract_text = Some(text.clone());
                }
                if in_published && let Some(y) = text.get(0..4).and_then(|s| s.parse::<i64>().ok())
                {
                    year = Some(y);
                }
                if in_name {
                    current_name = text;
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    if title.is_none() && authors.is_empty() {
        return None;
    }

    Some(pdf::RawMetadata {
        title,
        authors,
        journal: None,
        pub_year: year,
        doi: None,
        arxiv_id: None,
        abstract_text,
        keywords: Vec::new(),
        source: pdf::MetadataSource::Arxiv,
        body_text: None,
    })
}

pub(crate) fn apply_api_metadata(state: &mut AppState, meta: pdf::RawMetadata, doc_id: i64) {
    let is_api_source = matches!(
        meta.source,
        pdf::MetadataSource::Crossref | pdf::MetadataSource::Arxiv
    );
    let result = {
        if let Ok(conn) = state.db.lock() {
            if let Ok(Some(mut doc)) = documents::get_by_id(&conn, doc_id) {
                let mut changed = false;
                if is_api_source {
                    if let Some(ref t) = meta.title {
                        if !t.is_empty() && t != "Untitled" {
                            doc.title = t.clone();
                            changed = true;
                        }
                    }
                    if !meta.authors.is_empty() {
                        doc.authors = Some(meta.authors.join("; "));
                        changed = true;
                    }
                    if let Some(ref j) = meta.journal {
                        if !j.is_empty() {
                            doc.journal = Some(j.clone());
                            changed = true;
                        }
                    }
                    if let Some(y) = meta.pub_year {
                        doc.pub_year = Some(y);
                        changed = true;
                    }
                    if let Some(ref d) = meta.doi {
                        if doc.doi.is_none() {
                            doc.doi = Some(d.clone());
                            changed = true;
                        }
                    }
                    if let Some(ref a) = meta.abstract_text {
                        if !a.is_empty() {
                            doc.abstract_text = Some(a.clone());
                            changed = true;
                        }
                    }
                } else {
                    if (doc.title.is_empty() || doc.title == "Untitled")
                        && let Some(ref t) = meta.title
                    {
                        doc.title = t.clone();
                        changed = true;
                    }
                    let authors_empty =
                        doc.authors.as_deref().map_or(true, |a| a.trim().is_empty());
                    let authors_look_wrong = doc.authors.as_deref().map_or(false, |a| {
                        let a = a.trim();
                        !a.contains(' ') && a.len() <= 20
                    });
                    if !meta.authors.is_empty() && (authors_empty || authors_look_wrong) {
                        doc.authors = Some(meta.authors.join("; "));
                        changed = true;
                    }
                    if (doc.journal.is_none()
                        || doc.journal.as_deref().map_or(true, |j| j.trim().is_empty()))
                        && let Some(ref j) = meta.journal
                    {
                        doc.journal = Some(j.clone());
                        changed = true;
                    }
                    if doc.pub_year.is_none()
                        && let Some(y) = meta.pub_year
                    {
                        doc.pub_year = Some(y);
                        changed = true;
                    }
                    if doc.doi.is_none()
                        && let Some(ref d) = meta.doi
                    {
                        doc.doi = Some(d.clone());
                        changed = true;
                    }
                    if (doc.abstract_text.is_none()
                        || doc
                            .abstract_text
                            .as_deref()
                            .map_or(true, |a| a.trim().is_empty()))
                        && let Some(ref a) = meta.abstract_text
                    {
                        doc.abstract_text = Some(a.clone());
                        changed = true;
                    }
                }
                if changed {
                    match documents::update(&conn, &doc) {
                        Ok(()) => Some(Ok(())),
                        Err(e) => Some(Err(e.to_string())),
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    };

    match result {
        Some(Ok(())) => {
            state.reload_documents();
            if state.show_detail
                && state
                    .detail_doc
                    .as_ref()
                    .and_then(|d| d.id)
                    .map(|d_id| d_id == doc_id)
                    .unwrap_or(false)
            {
                state.load_detail();
            }
            state.finish_processing("API 메타데이터 보강 완료");
        }
        Some(Err(msg)) => state.finish_processing(&format!("API 저장 실패: {}", msg)),
        None => state.finish_processing("API: 보강할 필드 없음"),
    }
}
