fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        let mut res = winres::WindowsResource::new();
        res.set_icon("icon.ico");
        res.set("FileDescription", "MultiSerial Port Monitor");
        res.set("ProductName", "MultiSerial");
        res.compile().unwrap();
    }
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=icon.ico");
}
