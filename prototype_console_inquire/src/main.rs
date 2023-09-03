use std::fmt::Display;

use inquire::Select;

struct NextStepItem {
    summary: String,
}

impl Display for NextStepItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.summary)
    }
}

fn main() {
    const CARGO_PKG_VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");

    println!("Welcome to On-Purpose: Time Management Rethought");
    println!("This is the console prototype using the inquire package");
    println!("Version {}", CARGO_PKG_VERSION.unwrap_or("UNKNOWN"));

    let bullet_list = vec![
        NextStepItem {
            summary: String::from("Clean Dometic")
        },
        NextStepItem {
            summary: String::from("Fill out SafeAccess Health & Safety Invitation for RustConf 2023")
        },
    ];

    let selected = Select::new("Select one", bullet_list).prompt();

    let selected = selected.unwrap();

    println!("{} selected", selected);
}