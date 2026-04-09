fn main() {
    // 从环境变量获取版本号，默认为 Cargo.toml 中的版本
    let version = std::env::var("PORT_LENS_VERSION").unwrap_or_else(|_| {
        std::env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.1.0".to_string())
    });

    // 将版本号注入到编译时环境变量
    println!("cargo:rustc-env=PORT_LENS_BUILD_VERSION={version}");

    // 如果环境变量变化，重新编译
    println!("cargo:rerun-if-env-changed=PORT_LENS_VERSION");
}
