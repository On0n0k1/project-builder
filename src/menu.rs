use inquire::Select;
use serde::Deserialize;
use std::{default::Default, str::FromStr};
use strum::VariantNames;

use crate::projects::{Database, Deployment, Language, Platform, Projects, SearchParameter};

const CLEAR_STRING: &str = "(clear)";

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Menu {
    parameters: SearchParameter,
    projects: Projects,
}

impl Menu {
    pub fn new(projects: Projects) -> Self {
        let parameters = SearchParameter::default();
        Self {
            parameters,
            projects,
        }
    }
}

impl MenuExt for Menu {
    fn parameters(&self) -> &SearchParameter {
        &self.parameters
    }

    fn parameters_mut(&mut self) -> &mut SearchParameter {
        &mut self.parameters
    }

    fn projects(&self) -> &Projects {
        &self.projects
    }
}

pub const OPTIONS: [&str; 5] = ["Source", "Platform", "Language", "Database", "Deployment"];

fn parameters_update<A: FromStr + VariantNames>(parameter_name: &str) -> Option<A>
where
    <A as std::str::FromStr>::Err: std::fmt::Debug,
{
    let mut options: Vec<String> = A::VARIANTS
        .iter()
        .map(|option| option.to_string())
        .collect();
    options.push("(clear)".to_string());
    let selected: String = Select::new(
        &format!("Select which type of {parameter_name} to filter: "),
        options,
    )
    .prompt()
    .unwrap();
    if selected == CLEAR_STRING {
        return None;
    }
    let result: A = A::from_str(&selected).unwrap();
    Some(result)
}

pub trait MenuExt {
    fn parameters(&self) -> &SearchParameter;
    fn parameters_mut(&mut self) -> &mut SearchParameter;
    fn projects(&self) -> &Projects;

    fn sources(&self) -> Vec<String> {
        let mut sources: Vec<String> = Vec::with_capacity(self.projects().projects.len());
        self.projects()
            .projects
            .iter()
            .for_each(|project| sources.push(project.source.clone()));
        sources
    }

    fn menu(&mut self) -> Result<String, String> {
        loop {
            let mut options: Vec<String> = Vec::with_capacity(4 + self.sources().len());
            let database: String = self
                .parameters()
                .database
                .clone()
                .map(|database| database.to_string())
                .unwrap_or_default();

            let option = format!("Database: {database}");
            options.push(option);

            let deployment: String = self
                .parameters()
                .deployment
                .clone()
                .map(|deployment| deployment.to_string())
                .unwrap_or_default();
            let option = format!("Deployment: {deployment}");
            options.push(option);

            let language: String = self
                .parameters()
                .language
                .clone()
                .map(|language| language.to_string())
                .unwrap_or_default();
            let option = format!("Language: {language}");
            options.push(option);

            let platform: String = self
                .parameters()
                .platform
                .clone()
                .map(|platform| platform.to_string())
                .unwrap_or_default();
            let option = format!("Platform: {platform}");
            options.push(option);

            let search_parameter: &SearchParameter = self.parameters();
            for project in &self.projects().projects {
                if *search_parameter == *project {
                    options.push(project.source.clone());
                    continue;
                }
                println!("Comparison failed: \n{search_parameter:?}\nvs\n{project:?}\n");
            }
            let selected = Select::new("Select a project or Filter Search", options)
                .prompt()
                .unwrap();
            if selected.starts_with("Database:") {
                let database: Option<Database> = parameters_update::<Database>("Database");
                self.parameters_mut().database = database;
                continue;
            }

            if selected.starts_with("Deployment:") {
                let deployment: Option<Deployment> = parameters_update::<Deployment>("Deployment");
                self.parameters_mut().deployment = deployment;
                continue;
            }

            if selected.starts_with("Language:") {
                let language: Option<Language> = parameters_update::<Language>("Language");
                self.parameters_mut().language = language;
                continue;
            }

            if selected.starts_with("Platform:") {
                let platform: Option<Platform> = parameters_update::<Platform>("Platform");
                self.parameters_mut().platform = platform;
                continue;
            }
            return Ok(selected);
        }
    }
}
