fn main() {
    println!("cargo:rustc-link-lib=X11");
    println!("cargo:rustc-link-lib=Xfixes");
    println!("cargo:rustc-link-lib=Xi");
    println!("cargo:rustc-link-lib=Xcursor");

    if cfg!(target_os = "linux") {
        match env!("XDG_SESSION_TYPE") {
            "x11" => {
                println!("cargo::rustc-cfg=x11_impl");
            }
            "wayland" => {
                println!("cargo::rustc-cfg=wayland_impl");
            }
            _ => {}
        }
    }
}
