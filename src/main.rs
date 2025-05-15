#![windows_subsystem = "windows"]

mod singleton;

fn main() {
    // Singleton enforcement using portable interface
    if singleton::platform::SingletonGuard::acquire().is_none() {
        std::process::exit(0);
    }
    println!("Hello, world!");
}
