fn main() {
    println!("cargo:rerun-if-changed=../../assets/icon.ico");
    println!("cargo:rerun-if-env-changed=MOUSEMEETER_VERSION");

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        // Resource id 1 is loaded at runtime for the tray icon (see tray.rs).
        res.set_icon_with_id("../../assets/icon.ico", "1");
        if let Ok(version) = std::env::var("MOUSEMEETER_VERSION") {
            res.set("ProductVersion", &version);
        }
        res.compile().expect("failed to compile Windows resources");
    }
}
