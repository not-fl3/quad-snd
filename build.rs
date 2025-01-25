fn main() {
    let target = std::env::var("CARGO_CFG_TARGET_OS");

    if target.as_deref() == Ok("macos") || target.as_deref() == Ok("ios") {
        println!("cargo:rustc-link-lib=framework=CoreAudio");
        println!("cargo:rustc-link-lib=framework=AudioToolBox");
    }
}
