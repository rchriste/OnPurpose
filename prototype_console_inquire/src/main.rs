fn main() {
    const CARGO_PKG_VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");

    println!("Welcome to On-Purpose: Time Management Rethought");
    println!("This is the console prototype using the inquire package");
    println!("Version {}", CARGO_PKG_VERSION.unwrap_or("UNKNOWN"));
}