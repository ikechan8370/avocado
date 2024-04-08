use std::fs;
use std::path::{Path, PathBuf};

fn collect_proto_files<P: AsRef<Path>>(dir: P) -> Vec<PathBuf> {
    let mut proto_files = Vec::new();

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "proto") {
                proto_files.push(path);
            } else if path.is_dir() {
                proto_files.extend(collect_proto_files(path));
            }
        }
    }

    proto_files
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_files = collect_proto_files("kritor/protos");

    let proto_files_str = proto_files.iter()
        .map(|path| path.to_str().unwrap_or(""))
        .collect::<Vec<_>>();
    // let proto_dirs: Vec<PathBuf> = fs::read_dir("kritor/protos").unwrap().map(|d| d.unwrap().path()).collect();
    tonic_build::configure()
        .build_server(true)
        // .proto_path("kritor/protos")
        .compile(&proto_files_str, &["kritor/protos"])?;

    Ok(())
}
