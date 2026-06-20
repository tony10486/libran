use libran::pdf;
use std::path::Path;

fn main() {
    let path = Path::new("tmp/0711.0189v1.pdf");
    match pdf::process_file(path) {
        Ok(meta) => {
            println!("제목:   {:?}", meta.title);
            println!("저자:   {:?}", meta.authors);
            println!("arXiv:  {:?}", meta.arxiv_id);
            println!("DOI:    {:?}", meta.doi);
        }
        Err(e) => println!("오류: {}", e),
    }
}
