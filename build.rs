// build.rs for embedding Windows icons and version info using winres
#[cfg(target_os = "windows")]
fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_icon("assets/icons/syncthing_green.ico"); // Use green icon as main app icon
    res.set_icon_with_id("assets/icons/syncthing_green.ico", "syncthing_green");
    res.set_icon_with_id("assets/icons/syncthing_red.ico", "syncthing_red");
    res.set_manifest_file("assets/app.manifest"); // Optional: if you want a manifest
    res.compile().expect("Failed to compile Windows resources");
}

#[cfg(target_os = "linux")]
fn main() {}
