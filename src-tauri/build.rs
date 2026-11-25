fn main() {
    // Tauri needs the Common Controls v6 of Windows. Its build script embeds that manifest as a
    // resource of the app only, so a test binary of the core started without it and Windows
    // refused it with STATUS_ENTRYPOINT_NOT_FOUND. The linker embeds the same manifest
    // (`windows-app-manifest.xml`, the default of tauri-build 2.7.6) into every binary instead.
    let mut attributes = tauri_build::Attributes::new();
    let windows_msvc = std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows")
        && std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc");
    if windows_msvc {
        let manifest =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("windows-app-manifest.xml");
        println!("cargo:rerun-if-changed=windows-app-manifest.xml");
        println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
        println!("cargo:rustc-link-arg=/MANIFESTINPUT:{}", manifest.display());
        println!("cargo:rustc-link-arg=/WX");
        attributes = attributes
            .windows_attributes(tauri_build::WindowsAttributes::new_without_app_manifest());
    }
    if let Err(error) = tauri_build::try_build(attributes) {
        panic!("tauri-build failed: {error}");
    }
}
