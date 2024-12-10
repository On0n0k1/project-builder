use inquire::Select;

use crate::{
    error::Error,
    projects::{Projects, SearchParameter, SearchTree},
};

const CLEAR_STRING: &str = "(clear)";

#[derive(Debug, PartialEq, Eq)]
pub struct Menu {
    parameters: SearchParameter,
    tree: SearchTree,
}

impl Menu {
    pub fn new(projects: Projects) -> Result<Self, Error> {
        let tree: SearchTree = SearchTree::new(projects.projects)?;
        let parameters: SearchParameter = SearchParameter::new(tree.get_topics().to_owned());
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
    topic: &String,
    tree: &SearchTree,
    parameter: SearchParameter,
) -> Option<String> {
    println!("Function parameters_update\ntopic:{topic:?}\nparameter:{parameter:?}\n");
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
            for (topic, value) in self.parameters().topics.iter() {
                let value = match value {
                    None => "".to_string(),
                    Some(value) => value.to_string(),
                };
                let option = format!("{topic}: {value}");
                options.push(option);
            }

            let search_parameter: &SearchParameter = self.parameters();
            self.tree()
                .search(search_parameter)
                .iter()
                .for_each(|project| options.push(project.source.clone()));

            let selected = Select::new("Select a project or Filter Search", options)
                .prompt()
                .unwrap();
            let topic_name = selected
                .split(": ")
                .collect::<Vec<&str>>()
                .into_iter()
                .map(str::to_string)
                .collect::<Vec<String>>();
            if topic_name.len() > 1 {
                let tree = self.tree();
                let value: Option<String> =
                    parameters_update(&topic_name[0], tree, search_parameter.clone());
                self.parameters_mut()
                    .topics
                    .insert(topic_name[0].to_string(), value);
                continue;
            }

            return Ok(selected);
        }
    }
}
