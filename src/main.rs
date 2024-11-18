pub mod menu;
pub mod projects;
use std::{fs::File, io::Read};

use menu::Menu;
use menu::MenuExt;
use projects::Projects;

fn main() {
    let mut file = File::open("data.toml").unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();
    let projects: Projects = toml::from_str(&data).unwrap();
    let mut menu = Menu::new(projects);
    menu.menu().unwrap();
}
