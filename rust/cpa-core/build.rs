//! Links libxml2, used for the validation against the CPA schema.
//! macOS and Linux: the system libxml2 found with pkg-config. Windows: libxml2 from vcpkg
//! (`vcpkg install libxml2:x64-windows-static-md`).

fn main() {
    #[cfg(windows)]
    {
        vcpkg::Config::new().emit_includes(false).find_package("libxml2").expect("libxml2 from vcpkg");
    }
    #[cfg(not(windows))]
    {
        pkg_config::Config::new().probe("libxml-2.0").expect("libxml2 found with pkg-config");
    }
}
