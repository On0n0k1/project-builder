use std::collections::HashMap;

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
    pub platform: Option<String>,
    pub language: Option<String>,
    pub database: Option<String>,
    pub deployment: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub enum Topic {
    Platform,
    Language,
    Database,
    Deployment,
}

/// Search nodes for "platform", "language", "database" and "deployment" respectively
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchNode {
    /// For debugging. What topic is this Search node associated with
    pub topic: Topic,
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
        topic: &Topic,
    ) -> Vec<Arc<Project>> {
        let mut found: Vec<Arc<Project>> = Vec::with_capacity(projects.len());
        match &expected {
            None => found.extend(projects),
            Some(expected) => {
                for project in projects {
                    match topic {
                        Topic::Database => match &project.database {
                            None => {}
                            Some(database) => {
                                if expected.eq_ignore_ascii_case(database) {
                                    found.push(project);
                                }
                            }
                        },
                        Topic::Deployment => match &project.deployment {
                            None => {}
                            Some(deployment) => {
                                if expected.eq_ignore_ascii_case(deployment) {
                                    found.push(project);
                                }
                            }
                        },
                        Topic::Language => match &project.language {
                            None => {}
                            Some(language) => {
                                if expected.eq_ignore_ascii_case(language) {
                                    found.push(project);
                                }
                            }
                        },
                        Topic::Platform => match &project.platform {
                            None => {}
                            Some(platform) => {
                                if expected.eq_ignore_ascii_case(platform) {
                                    found.push(project);
                                }
                            }
                        },
                    }
                }
            }
        }
        found
    }

    pub fn create_children(
        topics: &[Topic],
        found: &[Arc<Project>],
        next: &mut HashMap<Option<String>, Arc<SearchNode>>,
    ) -> Result<(), Error> {
        if let Some(topic) = topics.first() {
            let search_node: SearchNode = SearchNode::new(found.to_vec(), topics.to_vec(), None)?;
            let search_node: Arc<SearchNode> = Arc::new(search_node);
            next.insert(None, search_node);
            // Creating next nodes
            for project in found {
                // topic is a single value for each node that doesn't change
                let expected = match topic {
                    Topic::Database => project.database.clone(),
                    Topic::Deployment => project.deployment.clone(),
                    Topic::Language => project.language.clone(),
                    Topic::Platform => project.platform.clone(),
                };
                // so there will be no inconsistencies on the key due to the selection above
                if next.contains_key(&expected) {
                    // no need to replace an existing key
                    continue;
                }
                let search_node: SearchNode =
                    SearchNode::new(found.to_vec(), topics.to_vec(), expected.clone())?;
                let search_node: Arc<SearchNode> = Arc::new(search_node);
                next.insert(expected, search_node);
            }
        }
        Ok(())
    }

    pub fn new(
        projects: Vec<Arc<Project>>,
        mut topics: Vec<Topic>,
        expected: Option<String>,
    ) -> Result<Self, Error> {
        let mut next: HashMap<Option<String>, Arc<SearchNode>> =
            HashMap::with_capacity(projects.len());
        if topics.is_empty() {
            return Err(Error::SearchNodeEmptyTopics);
        }
        let topic: Topic = topics.remove(0);
        let found: Vec<Arc<Project>> = Self::filter(projects, &expected, &topic);
        Self::create_children(&topics, &found, &mut next)?;
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
    topics: Vec<Topic>,
    projects: Vec<Arc<Project>>,
}

impl SearchTree {
    pub fn new(projects: Vec<Project>) -> Result<Self, Error> {
        let projects: Vec<Arc<Project>> = projects.into_iter().map(Arc::new).collect();
        let topics: Vec<Topic> = vec![
            Topic::Platform,
            Topic::Language,
            Topic::Database,
            Topic::Deployment,
        ];
        let mut next: HashMap<Option<String>, Arc<SearchNode>> =
            HashMap::with_capacity(projects.len());
        SearchNode::create_children(&topics, &projects, &mut next)?;
        Ok(Self {
            next,
            topics,
            projects,
        })
    }

    pub fn get_projects(&self) -> &[Arc<Project>] {
        &self.projects[..]
    }

    fn next_node(
        next: &HashMap<Option<String>, Arc<SearchNode>>,
        topic: &Topic,
        parameter: &SearchParameter,
    ) -> Option<Arc<SearchNode>> {
        let expected: Option<String> = match topic {
            Topic::Database => parameter.database.clone(),
            Topic::Deployment => parameter.deployment.clone(),
            Topic::Language => parameter.language.clone(),
            Topic::Platform => parameter.platform.clone(),
        };
        next.get(&expected).cloned()
    }

    pub fn search(&self, parameter: &SearchParameter) -> Vec<Arc<Project>> {
        let topics: &Vec<Topic> = &self.topics;
        // If there are no filters in the search parameters
        if *parameter == SearchParameter::default() {
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

    /// retrieve all possible alternatives for a specific topic
    pub fn retrieve_topic_options(
        &self,
        expected_topic: &Topic,
        mut parameter: SearchParameter,
    ) -> Vec<String> {
        // Must include all possible options for the topic that is being expected
        // So will filter None of it
        match expected_topic {
            Topic::Database => parameter.database = None,
            Topic::Deployment => parameter.deployment = None,
            Topic::Language => parameter.language = None,
            Topic::Platform => parameter.platform = None,
        };
        let topics: &Vec<Topic> = &self.topics;
        if *expected_topic == topics[0] {
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

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq)]
pub struct SearchParameter {
    pub platform: Option<String>,
    pub language: Option<String>,
    pub database: Option<String>,
    pub deployment: Option<String>,
}
