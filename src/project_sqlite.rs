use std::collections::{HashMap, HashSet};
use std::sync::mpsc::Iter;

use clap::builder::Str;
use serde::{Deserialize, Serialize};
use sqlite::Connection;
use sqlite::Error;
use sqlite::Value;

use crate::projects::SearchParameter;

#[derive(Debug, Default, Deserialize)]
pub struct Projects {
    pub projects: Vec<Project>,
}

pub struct Data {
    connection: Connection,
    topics: Vec<String>,
}

impl Data {
    pub fn new(projects: Vec<Project>) -> Self {
        let mut topics: HashSet<String> = HashSet::new();
        // println!("Starting connection...");
        let connection: Connection = sqlite::open(":memory:").unwrap();
        // println!("Checking projects...");
        for project in projects.iter() {
            for key in project.topics.keys() {
                if !topics.contains(key) {
                    topics.insert(key.clone());
                }
            }
        }
        let mut topics: Vec<String> = topics.into_iter().collect::<Vec<String>>();
        // println!("Found these unique keys: {topics:?}\n");
        let mut keys = topics
            .iter()
            .to_owned()
            .map(|key| format!("{key} TEXT"))
            .collect::<Vec<String>>()
            .join(", ");
        if !keys.is_empty() {
            keys = format!(", {keys}");
        }
        let query = format!("CREATE TABLE projects (source TEXT PRIMARY KEY{keys});");
        // println!("Running Query:\n{query}\n");
        connection.execute(query).unwrap();
        // println!("Query executed successfully");

        let mut keys = topics.to_vec().join(", ");
        if !keys.is_empty() {
            keys = format!(", {keys}");
        }
        for project in projects {
            let mut keys = "source".to_string();
            let mut values = format!("'{}'", project.source);
            for topic in topics.iter() {
                match project.topics.get(topic) {
                    None => {}
                    Some(value) => {
                        keys = format!("{keys}, {topic}");
                        values = format!("{values}, '{value}'");
                    }
                }
            }
            let query = format!("INSERT INTO projects ({keys}) VALUES ({values});");

            // println!("Running: \n{query}\n");
            connection.execute(query).unwrap();
        }

        // let query = "SELECT * FROM projects;";
        // topics.insert(0, "source".to_string());
        // for row in connection
        //     .prepare(query)
        //     .unwrap()
        //     .into_iter()
        //     .map(|row| row.unwrap())
        // {
        //     for (index, topic) in topics.iter().enumerate() {
        //         if row.contains(index) {
        //             let value: Result<&str, Error> = row.try_read::<&str, _>(index);
        //             if let Ok(value) = value {
        //                 println!("{topic}: {}", value);
        //             }
        //         }
        //     }
        //     println!("\n-------------------------------------\n");
        // }
        let output = Self { connection, topics };
        // println!("Retrieving topics for deployment: ");
        // let result = output.retrieve_topics(Default::default(), "deployment");
        // println!("Output: {result:?}");
        // println!("Retrieving topics for platform: ");
        // let result = output.retrieve_topics(Default::default(), "platform");
        // println!("Output: {result:?}");
        // println!("Retrieving topics for language: ");
        // let result = output.retrieve_topics(Default::default(), "language");
        // println!("Output: {result:?}");
        // println!("Retrieving topics for database: ");
        // let result = output.retrieve_topics(Default::default(), "database");
        // println!("Output: {result:?}");

        println!("Finished");
        let projects = output.search(&Default::default());
        println!("Projects is:\n{projects:#?}");
        output
    }

    fn retrieve_topics(
        &self,
        parameters: &HashMap<String, String>,
        target_topic: &str,
    ) -> Vec<String> {
        let topics = &self.topics;
        let mut sections = vec![format!("{} IS NOT NULL", target_topic.to_string())];
        for topic in topics {
            if parameters.is_empty() {
                // println!("Empty");
                break;
            }
            // println!("topic: {topic}");
            if parameters.contains_key(topic) {
                if topic.eq(target_topic) {
                    // println!("Target");
                } else {
                    // println!("Not Target");
                    let value = parameters.get(topic).unwrap();
                    let section = format!("{topic}={value}");
                    sections.push(section);
                };
            }
            // return topics.clone();
        }
        let sections = sections.join("\n    AND ");
        // let inner_query = format!("    SELECT * FROM projects\n    WHERE {sections}");
        // let full_query =
        //     format!("SELECT DISTINCT {target_topic}\nFROM (\n{inner_query}\n) subquery");
        let full_query = format!("SELECT DISTINCT {target_topic} FROM projects\n WHERE {sections}");
        // println!("Running: {full_query}");
        let mut output = Vec::with_capacity(topics.len());
        for row in self
            .connection
            .prepare(full_query)
            .unwrap()
            .into_iter()
            .map(|row| row.unwrap())
        {
            // Query will return single column rows, this probably isn't necessary
            for (index, _topic) in topics.iter().enumerate() {
                if row.contains(index) {
                    let value: Result<&str, Error> = row.try_read::<&str, _>(index);
                    // print!("{value} ");
                    if let Ok(value) = value {
                        // println!("{topic}: {}", value);
                        output.push(value.to_string());
                    }
                }
            }
            // println!("-----------------------------");
        }

        output
    }

    fn search(&self, parameters: &HashMap<String, String>) -> Vec<Project> {
        let query: &str = "SELECT * FROM projects;";
        let mut projects: Vec<Project> = Vec::new();
        for row in self
            .connection
            .prepare(query)
            .unwrap()
            .into_iter()
            .map(|row| row.unwrap())
        {
            // println!("Setting source for row:\n{row:?}");
            let source = row.try_read::<&str, _>(0).unwrap().to_string();
            // println!("Source is {source}");

            let topics = HashMap::new();
            let mut project: Project = Project { source, topics };
            // println!("Iterating through columns");
            for (index, topic) in self.topics.iter().enumerate() {
                if row.contains(index + 1) {
                    // println!("Checking column {} for topic {topic}", index + 1);
                    let value: Result<&str, Error> = row.try_read::<&str, usize>(index + 1);
                    if let Ok(value) = value {
                        // println!("Value is {value}");
                        project.topics.insert(topic.to_string(), value.to_string());
                    }
                } else {
                    break;
                }
            }
            println!("Inserting Project: {project:#?}");
            projects.push(project);
        }
        projects
    }
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct Project {
    pub source: String,
    #[serde(flatten)]
    pub topics: HashMap<String, String>,
}
