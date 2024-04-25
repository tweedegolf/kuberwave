use serde_derive::{Deserialize, Serialize};
use slugify::slugify;
use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Limits {
    pub ingresses: Option<String>,
    pub jobs: Option<String>,
    pub pods: Option<String>,
    pub services: Option<String>,
    pub cpu: Option<String>,
    pub memory: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Namespace {
    pub name: String,
    pub limits: Option<Limits>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum Permission {
    Write,
    Read,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum ClusterRole {
    /// You can do anything. (within that namespace and not the cluster iff normal rolebinding)
    All,
    /// Get, list and watch all resources, with the notable exception of secrets.
    View,
}

#[derive(Eq, PartialEq, Serialize, Deserialize, Debug, Clone, Copy)]
#[derive(Default)]
pub enum UserKind {
    /// A Google Cloud user account or Google IAM service account
    User,
    /// A kubernetes service account
    #[default]
    ServiceAccount,
}



impl Display for UserKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                UserKind::User => "User",
                UserKind::ServiceAccount => "ServiceAccount",
            }
        )
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum UserNames {
    Single(String),
    Multiple(Vec<String>),
}

impl UserNames {
    pub fn get_file_name(&self) -> String {
        match self {
            UserNames::Single(s) => slugify!(s),
            UserNames::Multiple(v) => {
                use itertools::Itertools;
                v.iter().map(|s| slugify!(s)).join("-")
            }
        }
    }

    pub fn single(&self) -> &String {
        match self {
            UserNames::Single(s) => s,
            _ => panic!("Invalid: cannot have array of usernames in this case"),
        }
    }

    pub fn vec(&self) -> Vec<&str> {
        match self {
            UserNames::Single(s) => vec![s],
            UserNames::Multiple(v) => v.iter().map(|s| s as &str).collect(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub name: UserNames,
    #[serde(default)]
    pub kind: UserKind,
    pub cluster_permissions: Option<Vec<ClusterRole>>,
    pub permissions: Option<HashMap<String, Vec<ClusterRole>>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClusterManifest {
    pub namespaces: Vec<Namespace>,
    pub users: Vec<User>,
}
