fn main() {
    // ggml-cpu (whisper.cpp) memakai API registry Windows untuk membaca info CPU,
    // yang berada di advapi32 dan tidak ditautkan otomatis oleh whisper-rs-sys.
    let windows = std::env::var("CARGO_CFG_TARGET_OS").is_ok_and(|v| v == "windows");
    let whisper = std::env::var_os("CARGO_FEATURE_WHISPER").is_some();
    if windows && whisper {
        println!("cargo:rustc-link-lib=advapi32");
    }
    println!("cargo:rerun-if-changed=build.rs");
}
