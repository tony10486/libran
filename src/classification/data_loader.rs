use anyhow::Result;
use rusqlite::Connection;

use super::msc::MscScheme;
use super::physh::PhyshScheme;
use super::scheme::{register_scheme, set_label_by_notation};
use super::udc::UdcScheme;

const UDC_CSV: &str = include_str!("../../assets/udc_top_ko.csv");
const PHYSH_CSV: &str = include_str!("../../assets/physh_ko.csv");
const MSC_CSV: &str = include_str!("../../assets/msc_ko.csv");

pub fn load_all_schemes(conn: &Connection) -> Result<()> {
    let udc = UdcScheme::new();
    let udc_id = register_scheme(conn, &udc)?;
    load_labels_from_csv(conn, udc_id, "ko", UDC_CSV, "bundle")?;

    let physh = PhyshScheme::new();
    let physh_id = register_scheme(conn, &physh)?;
    load_labels_from_csv(conn, physh_id, "ko", PHYSH_CSV, "bundle")?;

    let msc = MscScheme::new();
    let msc_id = register_scheme(conn, &msc)?;
    load_labels_from_csv(conn, msc_id, "ko", MSC_CSV, "bundle")?;

    Ok(())
}

fn load_labels_from_csv(
    conn: &Connection,
    scheme_id: i64,
    lang: &str,
    csv: &str,
    source: &str,
) -> Result<()> {
    let mut reader = csv::ReaderBuilder::new().from_reader(csv.as_bytes());
    for record in reader.records() {
        let record = record?;
        if record.len() < 2 {
            continue;
        }
        let notation = record.get(0).unwrap_or("").trim();
        let ko_label = record.get(1).unwrap_or("").trim();
        if notation.is_empty() || ko_label.is_empty() {
            continue;
        }
        let _ = set_label_by_notation(conn, scheme_id, notation, lang, ko_label, source);
    }
    Ok(())
}

pub fn resolve_label_for_node(
    conn: &Connection,
    scheme_id: i64,
    notation: &str,
    pref_label: &str,
    lang: &str,
) -> String {
    if lang == "en" {
        return pref_label.to_string();
    }

    let result: Option<String> = conn
        .query_row(
            "SELECT cl.label FROM classification_labels cl
             INNER JOIN classification_nodes cn ON cl.node_id = cn.id
             WHERE cn.scheme_id = ?1 AND cn.notation = ?2 AND cl.lang = ?3",
            rusqlite::params![scheme_id, notation, lang],
            |row| row.get(0),
        )
        .ok();

    result.unwrap_or_else(|| pref_label.to_string())
}

pub fn get_nodes_with_labels(
    conn: &Connection,
    scheme_id: i64,
    lang: &str,
) -> Vec<(String, String, Option<String>)> {
    let mut stmt = match conn.prepare(
        "SELECT cn.notation, cn.pref_label,
                (SELECT cl.label FROM classification_labels cl WHERE cl.node_id = cn.id AND cl.lang = ?2)
         FROM classification_nodes cn
         WHERE cn.scheme_id = ?1
         ORDER BY cn.sort_order, cn.notation",
    ) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    let rows = stmt.query_map(rusqlite::params![scheme_id, lang], |row| {
        let notation: String = row.get(0)?;
        let pref_label: String = row.get(1)?;
        let translated: Option<String> = row.get(2)?;
        Ok((notation, pref_label, translated))
    });

    match rows {
        Ok(rows) => rows.filter_map(|r| r.ok()).collect(),
        Err(_) => Vec::new(),
    }
}
