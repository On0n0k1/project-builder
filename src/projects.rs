use serde::Deserialize;
use strum_macros::{Display, EnumString, VariantNames};

#[derive(Debug, Default, Deserialize, PartialEq, Eq)]
pub struct Projects {
    pub projects: Vec<Project>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq)]
pub struct Project {
    pub source: String,
    pub platform: Option<Platform>,
    pub language: Option<Language>,
    pub database: Option<Database>,
    pub deployment: Option<Deployment>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct SearchParameter {
    pub platform: Option<Platform>,
    pub language: Option<Language>,
    pub database: Option<Database>,
    pub deployment: Option<Deployment>,
}

impl PartialEq<Project> for SearchParameter {
    fn eq(&self, other: &Project) -> bool {
        if self.platform.is_some() && (self.platform != other.platform) {
            println!("Platform is different: {self:?}");
            return false;
        }
        if self.language.is_some() && (self.language != other.language) {
            println!("Language is different: {self:?}");
            return false;
        }
        if self.database.is_some() && (self.database != other.database) {
            println!("Database is different: {self:?}");
            return false;
        }
        if self.deployment.is_some() && (self.deployment != other.deployment) {
            println!("Deployment is different: {self:?}");
            return false;
        }
        true
    }
}

impl PartialEq<SearchParameter> for Project {
    fn eq(&self, other: &SearchParameter) -> bool {
        other == self
    }
}

impl PartialEq<&SearchParameter> for Project {
    fn eq(&self, other: &&SearchParameter) -> bool {
        self == *other
    }
}

impl PartialEq<&Project> for SearchParameter {
    fn eq(&self, other: &&Project) -> bool {
        self == *other
    }
}

impl Eq for SearchParameter {}

#[derive(Clone, Debug, Deserialize, EnumString, Display, PartialEq, Eq, VariantNames)]
pub enum Platform {
    Aws,
    Shuttle,
    Vercel,
    GithubPages,
}

#[derive(Clone, Debug, Deserialize, EnumString, Display, PartialEq, Eq, VariantNames)]
pub enum Language {
    Rust(Option<FrameworkRust>),
    Javascript(Option<FrameworkJavascript>),
    CSharp,
    CPlusPlus,
    Python,
}

#[derive(Clone, Debug, Deserialize, EnumString, Display, PartialEq, Eq, VariantNames)]
pub enum FrameworkRust {
    Rocket,
    Actix,
}

#[derive(Clone, Debug, Deserialize, EnumString, Display, PartialEq, Eq, VariantNames)]
pub enum FrameworkJavascript {
    React,
    NodeJs,
}

#[derive(Clone, Debug, Deserialize, EnumString, Display, PartialEq, Eq, VariantNames)]
pub enum Database {
    PostgreSQL,
    DynamoDB,
    MySql,
    Redshift,
}

#[derive(Clone, Debug, Deserialize, EnumString, Display, PartialEq, Eq, VariantNames)]
pub enum Deployment {
    DockerCompose,
    Kubernetes,
    Terraform,
}
