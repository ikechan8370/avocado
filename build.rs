use std::fs;
use std::io::Write;
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

    println!("cargo:rerun-if-changed=kritor/protos");
    println!("cargo:rerun-if-changed=src/service/plugins");

    let proto_files = collect_proto_files("kritor/protos");

    let proto_files_str = proto_files.iter()
        .map(|path| path.to_str().unwrap_or(""))
        .collect::<Vec<_>>();
    // let proto_dirs: Vec<PathBuf> = fs::read_dir("kritor/protos").unwrap().map(|d| d.unwrap().path()).collect();
    let config = tonic_build::configure();
    let non_emum_message = vec![
        "kritor.common.TextElement",
        "kritor.common.AtElement",
        "kritor.common.FaceElement",
        "kritor.common.BubbleFaceElement",
        "kritor.common.ReplyElement",
        // "kritor.common.ImageElement",
        // "kritor.common.VoiceElement",
        // "kritor.common.VideoElement",
        "kritor.common.BasketballElement",
        "kritor.common.DiceElement",
        "kritor.common.RpsElement",
        "kritor.common.PokeElement",
        "kritor.common.CustomMusicData",
        // "kritor.common.MusicElement",
        "kritor.common.WeatherElement",
        "kritor.common.LocationElement",
        "kritor.common.ShareElement",
        "kritor.common.GiftElement",
        "kritor.common.MarketFaceElement",
        "kritor.common.ForwardElement",
        "kritor.common.ContactElement",
        "kritor.common.JsonElement",
        "kritor.common.XmlElement",
        "kritor.common.FileElement",
        "kritor.common.MarkdownElement",
        "kritor.common.ButtonActionPermission",
        "kritor.common.ButtonAction",
        "kritor.common.ButtonRender",
        "kritor.common.Button",
        "kritor.common.KeyboardRow",
        "kritor.common.KeyboardElement",
        "kritor.common.Sender",
        // "kritor.common.Scene",

    ];
    let config = non_emum_message.iter().fold(config, |config, message| {
        config.type_attribute(message, "#[derive(boa_engine::value::TryFromJs)]")
    });
    config
        // .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .build_server(true)
        // .proto_path("kritor/protos")
        .compile(&proto_files_str, &["kritor/protos"])?;


    // 加载插件
    let out_dirs = vec!["src/service/plugins", "src/service/plugins/default"];
    for out_dir in out_dirs {
        let dest_path = Path::new(&out_dir).join("mod.rs");
        let mut f = fs::File::create(&dest_path).unwrap();

        let paths = fs::read_dir(out_dir).unwrap();

        for path in paths {
            let path = path.unwrap().path();
            if path.is_dir() {
                if let Some(os_str) = path.file_name() {
                    if let Some(dir_name) = os_str.to_str() {
                        writeln!(f, "mod {};", dir_name).unwrap();
                    }
                }
            } else if let Some(extension) = path.extension() {
                if extension == "rs" {
                    if let Some(os_str) = path.file_stem() {
                        if let Some(file_name) = os_str.to_str() {
                            if file_name != "mod" && file_name != "main" {
                                writeln!(f, "mod {};", file_name).unwrap();
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
