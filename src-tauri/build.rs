fn main() {
    // On macOS, embed Info.plist (with NSMicrophoneUsageDescription) directly into
    // the binary's __TEXT,__info_plist section so the bare `tauri dev` binary — not
    // just the bundled .app — can request microphone access via TCC.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        let dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        println!(
            "cargo:rustc-link-arg=-Wl,-sectcreate,__TEXT,__info_plist,{}/Info.plist",
            dir
        );
        println!("cargo:rerun-if-changed=Info.plist");
    }
    tauri_build::build()
}
