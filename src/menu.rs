use inquire::Select;
use std::default::Default;

use crate::{
    error::Error,
    projects::{Projects, SearchParameter, SearchTree, Topic},
};

const CLEAR_STRING: &str = "(clear)";

#[derive(Debug, PartialEq, Eq)]
pub struct Menu {
    parameters: SearchParameter,
    tree: SearchTree,
}

impl Menu {
    pub fn new(projects: Projects) -> Result<Self, Error> {
        let parameters = SearchParameter::default();
        let tree = SearchTree::new(projects.projects)?;
        Ok(Self { parameters, tree })
    }
}

impl MenuExt for Menu {
    fn parameters(&self) -> &SearchParameter {
        &self.parameters
    }

    fn parameters_mut(&mut self) -> &mut SearchParameter {
        &mut self.parameters
    }

    fn tree(&self) -> &SearchTree {
        &self.tree
    }
}

pub const OPTIONS: [&str; 5] = ["Source", "Platform", "Language", "Database", "Deployment"];

fn parameters_update(
    topic: &Topic,
    tree: &SearchTree,
    parameter: SearchParameter,
) -> Option<String> {
    let mut options: Vec<String> = tree
        .retrieve_topic_options(topic, parameter)
        .into_iter()
        .collect();
    options.push(CLEAR_STRING.to_string());
    let selected: String = Select::new(
        &format!("Select which type of {topic:?} to filter: "),
        options,
    )
    .prompt()
    .unwrap();
    if selected == CLEAR_STRING {
        return None;
    }
    Some(selected)
}

pub trait MenuExt {
    fn parameters(&self) -> &SearchParameter;
    fn parameters_mut(&mut self) -> &mut SearchParameter;
    fn tree(&self) -> &SearchTree;

    fn sources(&self) -> Vec<String> {
        let projects = self.tree().get_projects();
        let mut sources: Vec<String> = Vec::with_capacity(projects.len());
        projects
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
            self.tree()
                .search(search_parameter)
                .iter()
                .for_each(|project| options.push(project.source.clone()));

            let selected = Select::new("Select a project or Filter Search", options)
                .prompt()
                .unwrap();
            let tree = self.tree();
            if selected.starts_with("Database:") {
                let database: Option<String> =
                    parameters_update(&Topic::Database, tree, search_parameter.clone());
                self.parameters_mut().database = database;
                continue;
            }

            if selected.starts_with("Deployment:") {
                let deployment: Option<String> =
                    parameters_update(&Topic::Deployment, tree, search_parameter.clone());
                self.parameters_mut().deployment = deployment;
                continue;
            }

            if selected.starts_with("Language:") {
                let language: Option<String> =
                    parameters_update(&Topic::Language, tree, search_parameter.clone());
                self.parameters_mut().language = language;
                continue;
            }

            if selected.starts_with("Platform:") {
                let platform: Option<String> =
                    parameters_update(&Topic::Platform, tree, search_parameter.clone());
                self.parameters_mut().platform = platform;
                continue;
            }
            return Ok(selected);
        }
    }
}
