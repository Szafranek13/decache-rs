fn main() {
    println!(
        "cargo:rustc-env=DECACHE_VERSION={}",
        if cfg!(debug_assertions) {
            "testing"
        } else {
            env!("CARGO_PKG_VERSION")
        }
    );
    println!(
        "cargo:rustc-env=BUILD_DATE={}",
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string()
    );
    println!(
        "cargo:rustc-env=BUILD_TARGET={}",
        std::env::var("TARGET").unwrap()
    );
}
