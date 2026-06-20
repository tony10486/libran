#[test]
fn diag_richard2017_text() {
    let path = std::path::PathBuf::from("tmp/[중요]richard2017.pdf");
    if !path.exists() {
        eprintln!("SKIP: PDF not found");
        return;
    }
    let text = libran::pdf::text::extract_text(&path).expect("extract_text");
    let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    eprintln!("=== {} non-empty lines, {} chars ===", lines.len(), text.len());
    for (i, line) in lines.iter().take(50).enumerate() {
        eprintln!("  L{:02}: {:?}", i, &line[..line.len().min(200)]);
    }
    eprintln!("\n=== Abstract search ===");
    for (i, line) in lines.iter().enumerate() {
        let low = line.trim().to_lowercase();
        if low.starts_with("abstract") {
            eprintln!("  L{:02}: {:?}", i, &line[..line.len().min(200)]);
            break;
        }
    }
    eprintln!("\n=== process_file result ===");
    let meta = libran::pdf::process_file(&path).expect("process_file");
    eprintln!("  title: {:?}", meta.title);
    eprintln!("  authors: {:?}", meta.authors);
    eprintln!("  journal: {:?}", meta.journal);
    eprintln!("  pub_year: {:?}", meta.pub_year);
}
