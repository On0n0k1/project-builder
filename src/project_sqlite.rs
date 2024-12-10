use std::collections::{HashMap, HashSet};

use clap::builder::Str;
use serde::{Deserialize, Serialize};
use sqlite::Connection;
use sqlite::Error;
use sqlite::Value;

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
        println!("Starting connection...");
        let connection: Connection = sqlite::open(":memory:").unwrap();
        println!("Checking projects...");
        for project in projects.iter() {
            for key in project.topics.keys() {
                if !topics.contains(key) {
                    topics.insert(key.clone());
                }
            }
        }
        let mut topics: Vec<String> = topics.into_iter().collect::<Vec<String>>();
        println!("Found these unique keys: {topics:?}\n");
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
        println!("Running Query:\n{query}\n");
        connection.execute(query).unwrap();
        println!("Query executed successfully");

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

            println!("Running: \n{query}\n");
            connection.execute(query).unwrap();
        }

        let query = "SELECT * FROM projects;";
        topics.insert(0, "source".to_string());
        for row in connection
            .prepare(query)
            .unwrap()
            .into_iter()
            .map(|row| row.unwrap())
        {
            for (index, topic) in topics.iter().enumerate() {
                if row.contains(index) {
                    let value: Result<&str, Error> = row.try_read::<&str, _>(index);
                    if let Ok(value) = value {
                        println!("{topic}: {}", value);
                    }
                }
            }
            println!("\n-------------------------------------\n");
        }
        println!("Finished");
        Self { connection, topics }
    }
    // fn retrieve_topics(&self, parameters: HashMap<String, String>) -> Vec<Project> {
    //     let topics = &self.topics;
    //     for topic in topics {
    //         if parameters.contains_key(topic) {
    //             let value = parameters.get(topic).unwrap();

    //         }
    //     }
    //     self.connection.execute(statement)
    //     Vec::new()
    // }
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct Project {
    source: String,
    #[serde(flatten)]
    topics: HashMap<String, String>,
}
