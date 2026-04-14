fn main() {
    // Link SDL2_ttf and dependencies from vcpkg when graphical feature is enabled
    #[cfg(feature = "graphical")]
    {
        println!("cargo:rustc-link-search=native=C:/src/vcpkg/installed/x64-windows/lib");
        println!("cargo:rustc-link-lib=static=SDL2_ttf");
        println!("cargo:rustc-link-lib=static=freetype");
        println!("cargo:rustc-link-lib=static=bz2");
        println!("cargo:rustc-link-lib=static=libpng16");
        println!("cargo:rustc-link-lib=static=zlib");
    }
}
