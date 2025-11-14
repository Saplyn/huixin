fn main() {
    println!("cargo:rustc-env=BUILD_INFO={}", build_info_string());
}

fn build_info_string() -> String {
    let pkg_name = env!("CARGO_PKG_NAME");
    let pkg_version = env!("CARGO_PKG_VERSION");
    let profile = std::env::var("PROFILE").unwrap_or("unknown".to_string());
    let built_time = chrono::Local::now()
        .format("%Y-%m-%d %H:%M:%S %Z")
        .to_string();
    format!("{pkg_name} v{pkg_version} built in {profile} mode at {built_time}")
}
