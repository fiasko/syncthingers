#[cfg(target_os = "windows")]
fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_icon("resources/icons/syncthingers.ico"); // This will be created later
    res.compile().unwrap();
}

#[cfg(not(target_os = "windows"))]
fn main() {
    // No resource embedding needed for non-Windows platforms
} 