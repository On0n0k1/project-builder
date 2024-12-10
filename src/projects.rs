use std::collections::{HashMap, HashSet};

use serde::Deserialize;
use std::sync::Arc;

use crate::error::Error;

#[derive(Debug, Default, Deserialize, PartialEq, Eq)]
pub struct Projects {
    pub projects: Vec<Project>,
}

// order of search is platform, language, database then deployment
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq)]
pub struct Project {
    pub source: String,
    #[serde(flatten)]
    pub topics: HashMap<String, String>,
}

/// Search nodes for "platform", "language", "database" and "deployment" respectively
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchNode {
    /// For debugging. What topic is this Search node associated with
    pub topic: String,
    /// For debugging. What search value this node represents
    pub expected: Option<String>,
    /// For searching. The values that this query so far represents at the end of a search (final node)
    pub found: Vec<Arc<Project>>,
    /// Next nodes according to the expected value for the next topic
    pub next: HashMap<Option<String>, Arc<SearchNode>>,
}

impl SearchNode {
    pub fn filter(
        projects: Vec<Arc<Project>>,
        expected: &Option<String>,
        topic: &str,
        node_depth: usize,
    ) -> Vec<Arc<Project>> {
        println!("Function SearchNode::filter\nnode_depth:{node_depth}\nexpected:{expected:?}\ntopic:{topic}\nprojects:{projects:?}\n");
        let mut found: Vec<Arc<Project>> = Vec::with_capacity(projects.len());
        match &expected {
            None => found.extend(projects),
            Some(expected) => {
                for project in projects {
                    if let Some(project_topic) = project.topics.get(topic) {
                        if expected.eq_ignore_ascii_case(project_topic) {
                            found.push(project);
                        }
                    }
                }
            }
        }
        found
    }

    pub fn create_children(
        topics: &[String],
        found: &[Arc<Project>],
        next: &mut HashMap<Option<String>, Arc<SearchNode>>,
        node_depth: usize,
    ) -> Result<(), Error> {
        println!(
            "Function SearchNode::create_children\nnode_depth:{node_depth}\ntopics:{topics:?}\nfound:{found:?}\n"
        );
        if let Some(topic) = topics.first() {
            let search_node: SearchNode =
                SearchNode::new(found.to_vec(), topics.to_vec(), None, node_depth)?;
            let search_node: Arc<SearchNode> = Arc::new(search_node);
            next.insert(None, search_node);
            // Creating next nodes
            for project in found {
                println!("Looping Project, node_depth {node_depth}: {project:?}\ntopic:{topic:?}\ntopics:{topics:?}\n");
                // topic is a single value for each node that doesn't change
                // let expected = match topic {
                //     Topic::Database => project.database.clone(),
                //     Topic::Deployment => project.deployment.clone(),
                //     Topic::Language => project.language.clone(),
                //     Topic::Platform => project.platform.clone(),
                // };
                // this is accessing out of bounds

                let expected = {
                    if !project.topics.contains_key(topic) {
                        // If project doesn't have the key, ignore it
                        None
                    } else {
                        Some(project.topics[topic].to_string())
                    }
                };
                // let expected: String = project.topics[topic].to_string();
                // // so there will be no inconsistencies on the key due to the selection above
                // let expected: Option<String> = Some(expected);
                if next.contains_key(&expected) {
                    // no need to replace an existing key
                    continue;
                }
                let search_node: SearchNode = SearchNode::new(
                    found.to_vec(),
                    topics.to_vec(),
                    expected.clone(),
                    node_depth,
                )?;
                let search_node: Arc<SearchNode> = Arc::new(search_node);
                next.insert(expected, search_node);
            }
        }
        Ok(())
    }

    pub fn new(
        projects: Vec<Arc<Project>>,
        mut topics: Vec<String>,
        expected: Option<String>,
        node_depth: usize,
    ) -> Result<Self, Error> {
        println!("Function SearchNode::new\nnode_depth:{node_depth}\ntopics: {topics:?}\nexpected: {expected:?}\nprojects:{projects:?}\n");
        let mut next: HashMap<Option<String>, Arc<SearchNode>> =
            HashMap::with_capacity(projects.len());
        if topics.is_empty() {
            return Err(Error::SearchNodeEmptyTopics);
        }
        let topic: String = topics.remove(0);
        let found: Vec<Arc<Project>> = Self::filter(projects, &expected, &topic, node_depth);
        Self::create_children(&topics, &found, &mut next, node_depth + 1)?;
        Ok(Self {
            topic,
            expected,
            found,
            next,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchTree {
    next: HashMap<Option<String>, Arc<SearchNode>>,
    topics: Vec<String>,
    projects: Vec<Arc<Project>>,
}

impl SearchTree {
    pub fn new(projects: Vec<Project>) -> Result<Self, Error> {
        println!("Function SearchTree::new\nprojects:{projects:?}\n");
        let projects: Vec<Arc<Project>> = projects.into_iter().map(Arc::new).collect();
        let mut topics: HashSet<String> = HashSet::new();
        for project in projects.iter() {
            for topic in project.topics.keys() {
                topics.insert(topic.to_string());
            }
        }
        let topics: Vec<String> = topics.into_iter().collect();
        let mut next: HashMap<Option<String>, Arc<SearchNode>> =
            HashMap::with_capacity(projects.len());
        SearchNode::create_children(&topics, &projects, &mut next, 0)?;
        Ok(Self {
            next,
            topics,
            projects,
        })
    }

    pub fn get_projects(&self) -> &[Arc<Project>] {
        &self.projects[..]
    }

    pub fn get_topics(&self) -> &[String] {
        &self.topics
    }

    fn next_node(
        next: &HashMap<Option<String>, Arc<SearchNode>>,
        topic: &String,
        parameter: &SearchParameter,
    ) -> Option<Arc<SearchNode>> {
        println!("Function next_node topic: {topic}\nparameter:{parameter:?}\n");
        let expected: Option<String> = parameter.topics[topic].to_owned();
        // let expected: Option<String> = match topic {
        //     Topic::Database => parameter.database.clone(),
        //     Topic::Deployment => parameter.deployment.clone(),
        //     Topic::Language => parameter.language.clone(),
        //     Topic::Platform => parameter.platform.clone(),
        // };
        next.get(&expected).cloned()
    }

    pub fn search(&self, parameter: &SearchParameter) -> Vec<Arc<Project>> {
        println!("Function search parameter:{parameter:?}");
        let topics: &Vec<String> = &self.topics;
        // If there are no filters in the search parameters
        if parameter.is_empty() {
            self.projects.clone()
        } else {
            let mut next: Arc<SearchNode> =
                Self::next_node(&self.next, &topics[0], parameter).unwrap();
            for topic in &topics[1..] {
                next = Self::next_node(&next.next, topic, parameter).unwrap();
            }
            next.found.clone()
        }
    }

    // This will not work. Will need to implement a different way of indexing the data that works for each unique topic
    // Maybe one tree for each unique topic?
    // Maybe one type of composite tree that can link per topic instead of project
    /// retrieve all possible alternatives for a specific topic
    pub fn retrieve_topic_options(
        &self,
        expected_topic: &String,
        mut parameter: SearchParameter,
    ) -> Vec<String> {
        println!("Function expected_topic: {expected_topic}\nparameter:{parameter:?}\n");
        // Must include all possible options for the topic that is being expected
        // So will filter None of it
        parameter.topics.insert(expected_topic.to_string(), None);
        // match expected_topic {
        //     Topic::Database => parameter.database = None,
        //     Topic::Deployment => parameter.deployment = None,
        //     Topic::Language => parameter.language = None,
        //     Topic::Platform => parameter.platform = None,
        // };
        let topics: &Vec<String> = &self.topics;
        if *expected_topic == topics[0] && parameter.is_empty() {
            let mut output: Vec<String> = Vec::with_capacity(self.next.len());
            self.next.keys().for_each(|key| match key {
                None => {}
                Some(value) => output.push(value.to_string()),
            });

            return output;
        }
        let mut next: Arc<SearchNode> =
            Self::next_node(&self.next, &topics[0], &parameter).unwrap();
        for topic in &topics[1..] {
            if *expected_topic == *topic {
                let mut output: Vec<String> = Vec::with_capacity(next.next.len());
                next.next.keys().for_each(|key| match key {
                    None => {}
                    Some(value) => output.push(value.to_string()),
                });
                return output;
            }
            next = Self::next_node(&next.next, topic, &parameter).unwrap();
        }
        unreachable!();
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct SearchParameter {
    #[serde(flatten)]
    pub topics: HashMap<String, Option<String>>,
}

impl SearchParameter {
    pub fn new(topic_list: Vec<String>) -> Self {
        let mut topics: HashMap<String, Option<String>> = HashMap::with_capacity(topic_list.len());
        for topic in topic_list {
            topics.insert(topic, None);
        }
        Self { topics }
    }
    pub fn is_empty(&self) -> bool {
        let mut is_empty: bool = true;
        self.topics.values().for_each(|value| {
            if value.is_some() {
                is_empty = false;
            }
        });
        is_empty
    }
}
