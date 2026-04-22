fn main() {
    println!("cargo:rerun-if-changed=icons");
    println!("cargo:rerun-if-changed=../static/icon.png");
    tauri_build::build()
}
