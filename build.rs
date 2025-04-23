fn main() {
    println!("cargo:rustc-link-search=native=C:/Users/loren/Documents/Progetti rust/tpd_shooter_sdl/libs/SDL2/lib/x64"); // o lib/x86 se compili a 32 bit
    println!("cargo:rustc-link-lib=SDL2");
    println!("cargo:rustc-link-lib=SDL2_ttf");
    println!("cargo:rustc-link-lib=SDL2_image");
}