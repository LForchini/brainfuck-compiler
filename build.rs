use platform_dirs::AppDirs;
use std::fs;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=profiles/*.json");

    let config_path = AppDirs::new(Some("bfc"), true).unwrap().config_dir;

    fs::create_dir_all(config_path.clone()).unwrap();

    for file in fs::read_dir("profiles").unwrap().flatten() {
        let mut target_path = config_path.clone();
        target_path.push(file.file_name());
        if !target_path.exists() {
            fs::copy(file.path(), target_path).unwrap();
        }
    }
}
