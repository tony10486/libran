use libran::db::documents::Document;
use libran::export::{ExportFormat, export};
use std::io::Cursor;

fn standard_test_document() -> Document {
    Document {
        id: Some(1),
        title: "Deep Learning for Natural Language Processing".to_string(),
        authors: Some("Smith, John; Lee, Jane".to_string()),
        journal: Some("Journal of AI Research".to_string()),
        conference: None,
        pub_year: Some(2023),
        doi: Some("10.1234/test.2023.001".to_string()),
        arxiv_id: Some("2301.12345".to_string()),
        abstract_text: Some("This paper presents a novel approach.".to_string()),
        keywords: Some("machine learning, NLP".to_string()),
        citation_key: Some("smith2023deep".to_string()),
        source: Some("manual".to_string()),
        rating: Some(5),
        volume: Some("42".to_string()),
        issue: Some("7".to_string()),
        page_start: Some("551".to_string()),
        page_end: Some("565".to_string()),
        publisher: Some("AI Press".to_string()),
        city: Some("Boston".to_string()),
        edition: Some("1st".to_string()),
        isbn: Some("978-0-123456-78-9".to_string()),
        issn: Some("1234-5678".to_string()),
        url: Some("https://example.com/paper".to_string()),
        accessed_date: Some("2024-01-15".to_string()),
        file_path: None,
        file_hash: None,
        reading_status: None,
        reading_progress: None,
        queue_position: None,
        item_type: "article".to_string(),
    }
}

fn run_export(format: ExportFormat, doc: &Document) -> String {
    let mut buf = Vec::new();
    export(&[doc.clone()], format, &mut Cursor::new(&mut buf)).unwrap();
    String::from_utf8(buf).unwrap()
}

#[test]
fn test_golden_bibtex() {
    let doc = standard_test_document();
    let out = run_export(ExportFormat::Bibtex, &doc);
    assert!(out.contains("@article {smith2023deep"), "BibTeX: {out}");
    assert!(
        out.contains("title     = {Deep Learning for Natural Language Processing}"),
        "BibTeX: {out}"
    );
    assert!(
        out.contains("author    = {Smith, John; Lee, Jane}"),
        "BibTeX: {out}"
    );
    assert!(
        out.contains("journal   = {Journal of AI Research}"),
        "BibTeX: {out}"
    );
    assert!(out.contains("year      = {2023}"), "BibTeX: {out}");
    assert!(
        out.contains("doi       = {10.1234/test.2023.001}"),
        "BibTeX: {out}"
    );
    assert!(out.contains("eprint    = {2301.12345}"), "BibTeX: {out}");
    assert!(
        out.contains("keywords  = {machine learning, NLP}"),
        "BibTeX: {out}"
    );
}

#[test]
fn test_golden_csl_json() {
    let doc = standard_test_document();
    let out = run_export(ExportFormat::CslJson, &doc);
    let parsed: serde_json::Value = serde_json::from_str(&out).expect("CSL JSON parses");
    assert!(parsed.is_array(), "CSL JSON should be an array: {out}");
    assert_eq!(parsed[0]["id"], "smith2023deep", "CSL JSON: {out}");
    assert_eq!(parsed[0]["type"], "article-journal", "CSL JSON: {out}");
    assert_eq!(
        parsed[0]["title"], "Deep Learning for Natural Language Processing",
        "CSL JSON: {out}"
    );
    assert_eq!(
        parsed[0]["container-title"], "Journal of AI Research",
        "CSL JSON: {out}"
    );
    assert_eq!(
        parsed[0]["issued"]["date-parts"][0][0], 2023,
        "CSL JSON: {out}"
    );
    assert_eq!(parsed[0]["doi"], "10.1234/test.2023.001", "CSL JSON: {out}");
    assert_eq!(parsed[0]["author"][0]["family"], "Smith", "CSL JSON: {out}");
    assert_eq!(parsed[0]["author"][0]["given"], "John", "CSL JSON: {out}");
    assert_eq!(parsed[0]["author"][1]["family"], "Lee", "CSL JSON: {out}");
}

#[test]
fn test_golden_ris() {
    let doc = standard_test_document();
    let out = run_export(ExportFormat::Ris, &doc);
    assert!(out.contains("TY  - JOUR"), "RIS: {out}");
    assert!(
        out.contains("TI  - Deep Learning for Natural Language Processing"),
        "RIS: {out}"
    );
    assert!(out.contains("AU  - Smith, John"), "RIS: {out}");
    assert!(out.contains("AU  - Lee, Jane"), "RIS: {out}");
    assert!(out.contains("PY  - 2023"), "RIS: {out}");
    assert!(out.contains("JO  - Journal of AI Research"), "RIS: {out}");
    assert!(out.contains("DO  - 10.1234/test.2023.001"), "RIS: {out}");
    assert!(out.contains("VL  - 42"), "RIS: {out}");
    assert!(out.contains("IS  - 7"), "RIS: {out}");
    assert!(out.contains("SP  - 551"), "RIS: {out}");
    assert!(out.contains("EP  - 565"), "RIS: {out}");
    assert!(out.contains("ER  - "), "RIS: {out}");
}

#[test]
fn test_golden_csv() {
    let doc = standard_test_document();
    let out = run_export(ExportFormat::Csv, &doc);
    let header = "id,title,authors,journal,conference,pub_year,doi,arxiv_id,abstract,keywords,citation_key,source,rating";
    assert!(out.starts_with(header), "CSV header missing: {out}");
    assert!(
        out.contains("Deep Learning for Natural Language Processing"),
        "CSV: {out}"
    );
    assert!(out.contains("Smith, John; Lee, Jane"), "CSV: {out}");
    assert!(out.contains("Journal of AI Research"), "CSV: {out}");
    assert!(out.contains("2023"), "CSV: {out}");
    assert!(out.contains("10.1234/test.2023.001"), "CSV: {out}");
    assert!(out.contains("smith2023deep"), "CSV: {out}");
    assert!(out.contains("2301.12345"), "CSV: {out}");
}

#[test]
fn test_golden_mods() {
    let doc = standard_test_document();
    let out = run_export(ExportFormat::Mods, &doc);
    assert!(
        out.contains("xmlns=\"http://www.loc.gov/mods/v3\""),
        "MODS namespace: {out}"
    );
    assert!(out.contains("version=\"3.7\""), "MODS version attr: {out}");
    assert!(out.contains("<mods version=\"3.7\">"), "MODS: {out}");
    assert!(out.contains("<titleInfo>"), "MODS: {out}");
    assert!(
        out.contains("<title>Deep Learning for Natural Language Processing</title>"),
        "MODS: {out}"
    );
    assert!(out.contains("<name type=\"personal\">"), "MODS: {out}");
    assert!(
        out.contains("<namePart>Smith, John</namePart>"),
        "MODS: {out}"
    );
    assert!(
        out.contains("<namePart>Lee, Jane</namePart>"),
        "MODS: {out}"
    );
    assert!(
        out.contains("<identifier type=\"doi\">10.1234/test.2023.001</identifier>"),
        "MODS: {out}"
    );
    assert!(
        out.contains("<identifier type=\"arxiv\">2301.12345</identifier>"),
        "MODS: {out}"
    );
    assert!(out.contains("<dateIssued>2023</dateIssued>"), "MODS: {out}");
    assert!(out.contains("<detail type=\"volume\">"), "MODS: {out}");
    assert!(out.contains("<number>42</number>"), "MODS: {out}");
    assert!(out.contains("<detail type=\"issue\">"), "MODS: {out}");
    assert!(out.contains("<number>7</number>"), "MODS: {out}");
    assert!(out.contains("<extent unit=\"pages\">"), "MODS: {out}");
    assert!(out.contains("<start>551</start>"), "MODS: {out}");
    assert!(out.contains("<end>565</end>"), "MODS: {out}");
}

#[test]
fn test_golden_bibliontology_rdf() {
    let doc = standard_test_document();
    let out = run_export(ExportFormat::BibliontologyRdf, &doc);
    assert!(out.contains("@prefix bibo:"), "Bibliontology RDF: {out}");
    assert!(
        out.contains("bibo:AcademicArticle"),
        "Bibliontology RDF: {out}"
    );
    assert!(out.contains("dcterms:title"), "Bibliontology RDF: {out}");
    assert!(out.contains("foaf:Person"), "Bibliontology RDF: {out}");
    assert!(
        out.contains("foaf:surname \"Smith\""),
        "Bibliontology RDF: {out}"
    );
    assert!(
        out.contains("foaf:surname \"Lee\""),
        "Bibliontology RDF: {out}"
    );
    assert!(out.contains("bibo:doi"), "Bibliontology RDF: {out}");
    assert!(out.contains("dcterms:isPartOf"), "Bibliontology RDF: {out}");
    assert!(
        out.contains("dcterms:title \"Deep Learning for Natural Language Processing\""),
        "Bibliontology RDF: {out}"
    );
}

#[test]
fn test_golden_bookmarks() {
    let doc = standard_test_document();
    let out = run_export(ExportFormat::Bookmarks, &doc);
    assert!(
        out.contains("<!DOCTYPE NETSCAPE-Bookmark-file-1>"),
        "Bookmarks: {out}"
    );
    assert!(
        out.contains("HREF=\"https://doi.org/10.1234/test.2023.001\""),
        "Bookmarks: {out}"
    );
    assert!(
        out.contains("Deep Learning for Natural Language Processing"),
        "Bookmarks: {out}"
    );
    assert!(out.contains("Smith, J., &amp; Lee, J."), "Bookmarks: {out}");
    assert!(
        out.contains("ADD_DATE=\"2023-01-01T00:00:00Z\""),
        "Bookmarks: {out}"
    );
    assert!(
        out.contains("<DD>Smith, John; Lee, Jane"),
        "Bookmarks: {out}"
    );
}

#[test]
fn test_golden_cff() {
    let doc = standard_test_document();
    let out = run_export(ExportFormat::Cff, &doc);
    assert!(out.contains("cff-version: \"1.2.0\""), "CFF: {out}");
    assert!(out.contains("message:"), "CFF: {out}");
    assert!(out.contains("title:"), "CFF: {out}");
    assert!(out.contains("authors:"), "CFF: {out}");
    assert!(out.contains("references:"), "CFF: {out}");
    assert!(out.contains("type: article"), "CFF: {out}");
    assert!(
        out.contains("title: \"Deep Learning for Natural Language Processing\""),
        "CFF: {out}"
    );
    assert!(out.contains("family-names: \"Smith\""), "CFF: {out}");
    assert!(out.contains("doi: \"10.1234/test.2023.001\""), "CFF: {out}");
}

#[test]
fn test_golden_cff_references() {
    let doc = standard_test_document();
    let out = run_export(ExportFormat::CffReferences, &doc);
    assert!(out.contains("references:"), "CFF References: {out}");
    assert!(out.contains("type: article"), "CFF References: {out}");
    assert!(
        out.contains("title: \"Deep Learning for Natural Language Processing\""),
        "CFF References: {out}"
    );
    assert!(out.contains("authors:"), "CFF References: {out}");
    assert!(
        out.contains("family-names: \"Smith\""),
        "CFF References: {out}"
    );
    assert!(
        out.contains("given-names: \"John\""),
        "CFF References: {out}"
    );
    assert!(
        out.contains("family-names: \"Lee\""),
        "CFF References: {out}"
    );
    assert!(out.contains("year: 2023"), "CFF References: {out}");
    assert!(
        out.contains("doi: \"10.1234/test.2023.001\""),
        "CFF References: {out}"
    );
    assert!(out.contains("volume: \"42\""), "CFF References: {out}");
    assert!(out.contains("pages: \"551-565\""), "CFF References: {out}");
    assert!(
        !out.contains("cff-version:"),
        "CFF References should not have top-level fields: {out}"
    );
}

#[test]
fn test_golden_coins() {
    let doc = standard_test_document();
    let out = run_export(ExportFormat::Coins, &doc);
    assert!(out.contains("<span class=\"Z3988\""), "COinS: {out}");
    assert!(out.contains("ctx_ver=Z39.88-2004"), "COinS: {out}");
    assert!(out.contains("rft.genre=article"), "COinS: {out}");
    assert!(out.contains("rft.atitle="), "COinS: {out}");
    assert!(out.contains("rft.aulast=Smith"), "COinS: {out}");
    assert!(out.contains("rft.aufirst=John"), "COinS: {out}");
    assert!(out.contains("rft.volume=42"), "COinS: {out}");
    assert!(out.contains("rft.issue=7"), "COinS: {out}");
    assert!(
        out.contains("rft_id=info:doi/10.1234%2Ftest.2023.001"),
        "COinS: {out}"
    );
}

#[test]
fn test_golden_endnote_xml() {
    let doc = standard_test_document();
    let out = run_export(ExportFormat::EndnoteXml, &doc);
    assert!(out.contains("<xml>"), "EndNote XML: {out}");
    assert!(out.contains("<records>"), "EndNote XML: {out}");
    assert!(out.contains("<record>"), "EndNote XML: {out}");
    assert!(
        out.contains("<ref-type name=\"Journal Article\">17</ref-type>"),
        "EndNote XML: {out}"
    );
    assert!(
        out.contains("<style face=\"normal\" font=\"default\" size=\"100%\">"),
        "EndNote XML: {out}"
    );
    assert!(
        out.contains("Deep Learning for Natural Language Processing"),
        "EndNote XML: {out}"
    );
    assert!(
        out.contains("<author><style face=\"normal\" font=\"default\" size=\"100%\">Smith, John</style></author>"),
        "EndNote XML: {out}"
    );
    assert!(
        out.contains("<electronic-resource-num><style face=\"normal\" font=\"default\" size=\"100%\">10.1234/test.2023.001</style></electronic-resource-num>"),
        "EndNote XML: {out}"
    );
}

#[test]
fn test_golden_refer_bibix() {
    let doc = standard_test_document();
    let out = run_export(ExportFormat::ReferBibix, &doc);
    assert!(out.contains("%0 Journal Article"), "Refer/BibIX: {out}");
    assert!(out.contains("%A Smith, John"), "Refer/BibIX: {out}");
    assert!(out.contains("%A Lee, Jane"), "Refer/BibIX: {out}");
    assert!(
        out.contains("%T Deep Learning for Natural Language Processing"),
        "Refer/BibIX: {out}"
    );
    assert!(
        out.contains("%J Journal of AI Research"),
        "Refer/BibIX: {out}"
    );
    assert!(out.contains("%D 2023"), "Refer/BibIX: {out}");
    assert!(out.contains("%V 42"), "Refer/BibIX: {out}");
    assert!(out.contains("%N 7"), "Refer/BibIX: {out}");
    assert!(out.contains("%P 551-565"), "Refer/BibIX: {out}");
    assert!(
        out.contains("%R 10.1234/test.2023.001"),
        "Refer/BibIX: {out}"
    );
}

#[test]
fn test_golden_refworks_tagged() {
    let doc = standard_test_document();
    let out = run_export(ExportFormat::RefworksTagged, &doc);
    assert!(out.contains("RT Journal Article"), "RefWorks Tagged: {out}");
    assert!(out.contains("A1 Smith,John"), "RefWorks Tagged: {out}");
    assert!(out.contains("A1 Lee,Jane"), "RefWorks Tagged: {out}");
    assert!(
        out.contains("T1 Deep Learning for Natural Language Processing"),
        "RefWorks Tagged: {out}"
    );
    assert!(
        out.contains("JF Journal of AI Research"),
        "RefWorks Tagged: {out}"
    );
    assert!(out.contains("YR 2023"), "RefWorks Tagged: {out}");
    assert!(out.contains("VO 42"), "RefWorks Tagged: {out}");
    assert!(out.contains("IS 7"), "RefWorks Tagged: {out}");
    assert!(out.contains("SP 551"), "RefWorks Tagged: {out}");
    assert!(out.contains("OP 565"), "RefWorks Tagged: {out}");
    assert!(
        out.contains("DO 10.1234/test.2023.001"),
        "RefWorks Tagged: {out}"
    );
}

#[test]
fn test_golden_evernote_export() {
    let doc = standard_test_document();
    let out = run_export(ExportFormat::EvernoteExport, &doc);
    assert!(out.contains("<en-export "), "Evernote Export: {out}");
    assert!(out.contains("</en-export>"), "Evernote Export: {out}");
    assert!(out.contains("<note>"), "Evernote Export: {out}");
    assert!(out.contains("</note>"), "Evernote Export: {out}");
    assert!(
        out.contains("<title>Deep Learning for Natural Language Processing</title>"),
        "Evernote Export: {out}"
    );
    assert!(out.contains("<content><![CDATA["), "Evernote Export: {out}");
    assert!(out.contains("<en-note>"), "Evernote Export: {out}");
    assert!(
        out.contains("Smith, J., & Lee, J. (2023). Deep Learning for Natural Language Processing."),
        "Evernote Export: {out}"
    );
    assert!(
        out.contains("<source-url>https://doi.org/10.1234/test.2023.001</source-url>"),
        "Evernote Export: {out}"
    );
    assert!(
        out.contains("<author>Smith, John; Lee, Jane</author>"),
        "Evernote Export: {out}"
    );
}

#[test]
fn test_golden_tei() {
    let doc = standard_test_document();
    let out = run_export(ExportFormat::Tei, &doc);
    assert!(
        out.contains("<TEI xmlns=\"http://www.tei-c.org/ns/1.0\">"),
        "TEI: {out}"
    );
    assert!(out.contains("<biblStruct>"), "TEI: {out}");
    assert!(out.contains("<analytic>"), "TEI: {out}");
    assert!(out.contains("<monogr>"), "TEI: {out}");
    assert!(
        out.contains("<title level=\"a\">Deep Learning for Natural Language Processing</title>"),
        "TEI: {out}"
    );
    assert!(
        out.contains("<title level=\"j\">Journal of AI Research</title>"),
        "TEI: {out}"
    );
    assert!(out.contains("<surname>Smith</surname>"), "TEI: {out}");
    assert!(out.contains("<forename>John</forename>"), "TEI: {out}");
    assert!(out.contains("<surname>Lee</surname>"), "TEI: {out}");
    assert!(
        out.contains("<idno type=\"DOI\">10.1234/test.2023.001</idno>"),
        "TEI: {out}"
    );
    assert!(
        out.contains("<biblScope unit=\"volume\">42</biblScope>"),
        "TEI: {out}"
    );
    assert!(
        out.contains("<biblScope unit=\"page\" from=\"551\" to=\"565\">551-565</biblScope>"),
        "TEI: {out}"
    );
}

#[test]
fn test_golden_wikidata_qs() {
    let doc = standard_test_document();
    let out = run_export(ExportFormat::WikidataQuickStatements, &doc);
    assert!(out.contains("CREATE"), "Wikidata QS: {out}");
    assert!(out.contains("LAST\tP31\tQ13442814"), "Wikidata QS: {out}");
    assert!(
        out.contains("LAST\tLen\t\"Deep Learning for Natural Language Processing\""),
        "Wikidata QS: {out}"
    );
    assert!(
        out.contains("LAST\tP1476\ten:\"Deep Learning for Natural Language Processing\""),
        "Wikidata QS: {out}"
    );
    assert!(
        out.contains("LAST\tP2093\t\"Smith, John\"\tP1545\t\"1\""),
        "Wikidata QS: {out}"
    );
    assert!(
        out.contains("LAST\tP2093\t\"Lee, Jane\"\tP1545\t\"2\""),
        "Wikidata QS: {out}"
    );
    assert!(
        out.contains("LAST\tP577\t+2023-01-01T00:00:00Z/9"),
        "Wikidata QS: {out}"
    );
    assert!(
        out.contains("LAST\tP356\t\"10.1234/TEST.2023.001\""),
        "Wikidata QS: {out}"
    );
    assert!(
        out.contains("LAST\tP818\t\"2301.12345\""),
        "Wikidata QS: {out}"
    );
    assert!(
        out.contains("LAST\tP1433\ten:\"Journal of AI Research\""),
        "Wikidata QS: {out}"
    );
}
