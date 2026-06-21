use std::io::Write;

fn main() {
    let dir = std::path::PathBuf::from("tmp");
    let entries: Vec<_> = std::fs::read_dir(&dir)
        .expect("read_dir")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "pdf").unwrap_or(false))
        .collect();

    let mut f = std::fs::File::create("/tmp/all_pdf_diag.txt").unwrap();

    for entry in &entries {
        let path = entry.path();
        let name = path.file_name().unwrap().to_string_lossy();
        writeln!(f, "\n{}", "=".repeat(60)).unwrap();
        writeln!(f, "FILE: {}", name).unwrap();
        writeln!(f, "{}", "=".repeat(60)).unwrap();

        let meta = match libran::pdf::process_file(&path) {
            Ok(m) => m,
            Err(e) => {
                writeln!(f, "  ERROR: {}", e).unwrap();
                continue;
            }
        };

        writeln!(f, "  title: {:?}", meta.title).unwrap();
        writeln!(f, "  authors: {:?}", meta.authors).unwrap();
        writeln!(f, "  journal: {:?}", meta.journal).unwrap();
        writeln!(f, "  pub_year: {:?}", meta.pub_year).unwrap();
        writeln!(f, "  doi: {:?}", meta.doi).unwrap();
        writeln!(f, "  arxiv_id: {:?}", meta.arxiv_id).unwrap();

        if let Ok(text) = libran::pdf::text::extract_text(&path) {
            let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
            writeln!(f, "  text_lines: {}", lines.len()).unwrap();
            for (i, line) in lines.iter().take(30).enumerate() {
                writeln!(f, "  L{:02}: {:?}", i, &line[..line.len().min(120)]).unwrap();
            }

            // Show abstract marker detection
            let abstract_idx = libran::pdf::heuristic::find_abstract_marker_pub(&lines);
            writeln!(f, "  abstract_marker_idx: {:?}", abstract_idx).unwrap();
        }
    }
}
