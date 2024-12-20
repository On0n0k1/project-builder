pub mod error;
pub mod menu;
// pub mod project_new;
pub mod project_sqlite;
pub mod projects;
use menu::Menu;
use menu::MenuExt;
use project_sqlite::{Data, Projects};
// use projects::Projects;
use std::{fs::File, io::Read};

fn main() {
    let mut file: File = File::open("data.toml").unwrap();
    let mut data: String = String::new();
    file.read_to_string(&mut data).unwrap();
    let projects: Projects = toml::from_str(&data).unwrap();
    // let mut menu: Menu = Menu::new(projects).unwrap();
    // menu.menu().unwrap();
    let data = Data::new(projects.projects);
}
