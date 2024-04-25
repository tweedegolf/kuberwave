use askama::Template;

use crate::cluster::types::*;
use crate::resourcefile::{Resourcefile, Resourceproto};

#[derive(Template)]
#[template(path = "cluster/namespace.yml")]
pub struct NamespaceTemplate<'a> {
    name: &'a str,
}

impl<'a> NamespaceTemplate<'a> {
    pub fn instantiate(ns: &'a Namespace) -> Resourcefile {
        let mut rsrc = (Resourceproto {
            name: &format!("namespace-{}.yml", &ns.name),
            prototype: Self { name: &ns.name },
        })
        .render();

        rsrc.append(ResourcequotaTemplate::instantiate(&ns.limits, &ns.name));

        rsrc
    }
}

#[derive(Template)]
#[template(path = "cluster/serviceaccount.yml")]
pub struct ServiceAccountTemplate<'a> {
    name: &'a str,
}

impl<'a> ServiceAccountTemplate<'a> {
    pub fn instantiate(u: &'a User) -> Resourcefile {
        (Resourceproto {
            name: &format!("sa-{}.yml", &u.name.single()),
            prototype: Self {
                name: u.name.single(),
            },
        })
        .render()
    }
}

#[derive(Template)]
#[template(path = "cluster/clusterrolebinding.yml")]
pub struct ClusterRolebindingTemplate<'a> {
    name: &'a str,
    rolename: &'a str,
    usernames: Vec<&'a str>,
    kind: UserKind,
}

impl<'a> ClusterRolebindingTemplate<'a> {
    pub fn instantiate(
        permission: &ClusterRole,
        username: &'a UserNames,
        kind: UserKind,
    ) -> Resourcefile {
        let rolename: &str = match permission {
            ClusterRole::All => "kuberwave-all",
            ClusterRole::View => "kuberwave-view",
        };

        let userfilename = match kind {
            UserKind::ServiceAccount => username.single().clone(),
            UserKind::User => username.get_file_name(),
        };

        let name = &format!("{}-{}", rolename, userfilename);

        (Resourceproto {
            name: &format!("clusterrolebinding-{}.yml", name),
            prototype: ClusterRolebindingTemplate {
                name,
                rolename,
                usernames: username.vec(),
                kind,
            },
        })
        .render()
    }
}

#[derive(Template)]
#[template(path = "cluster/rolebinding.yml")]
pub struct RolebindingTemplate<'a> {
    name: &'a str,
    namespace: &'a str,
    bindingkind: &'a str,
    rolename: &'a str,
    usernames: Vec<&'a str>,
    subjectkind: UserKind,
}

impl<'a> RolebindingTemplate<'a> {
    pub fn instantiate(
        permission: &ClusterRole,
        namespace: &'a str,
        username: &'a UserNames,
        kind: UserKind,
    ) -> Resourcefile {
        let rolename: &str = match permission {
            ClusterRole::All => "kuberwave-all",
            ClusterRole::View => "kuberwave-view",
        };

        let userfilename = match kind {
            UserKind::ServiceAccount => username.single().clone(),
            UserKind::User => username.get_file_name(),
        };

        let name = &format!("{}-{}-{}", namespace, rolename, userfilename);

        (Resourceproto {
            name: &format!("rolebinding-{}.yml", name),
            prototype: RolebindingTemplate {
                name,
                namespace,
                bindingkind: "ClusterRole",
                rolename,
                usernames: username.vec(),
                subjectkind: kind,
            },
        })
        .render()
    }
}

#[derive(Template)]
#[template(path = "cluster/resourcequota.yml")]
pub struct ResourcequotaTemplate<'a> {
    name: &'a str,
    namespace: &'a str,
    ingresses: &'a str,
    jobs: &'a str,
    pods: &'a str,
    services: &'a str,
    cpu: &'a str,
    memory: &'a str,
}

fn flatten<T>(x: &Option<Option<T>>) -> &Option<T> {
    match x {
        Some(x) => x,
        None => &None,
    }
}

fn unwrap_or_else_opt_string<'a, 'b: 'a>(
    opt: &'a Option<Option<&'a String>>,
    default: &'b str,
) -> &'a str {
    flatten(opt)
        .as_ref()
        .map(|str| str.as_str())
        .unwrap_or(default)
}

impl<'a> ResourcequotaTemplate<'a> {
    pub fn instantiate(limits: &'a Option<Limits>, namespace: &'a str) -> Resourcefile {
        let f = unwrap_or_else_opt_string;

        let limits = limits.as_ref();

        (Resourceproto {
            name: &format!("resourcequota-{}.yml", namespace),
            prototype: ResourcequotaTemplate {
                name: "kuberwave-resource-quotas",
                namespace,
                ingresses: f(&limits.map(|l| l.ingresses.as_ref()), "20"),
                jobs: f(&limits.map(|l| l.jobs.as_ref()), "100"),
                pods: f(&limits.map(|l| l.pods.as_ref()), "25"),
                services: f(&limits.map(|l| l.services.as_ref()), "25"),
                cpu: f(&limits.map(|l| l.cpu.as_ref()), "2000m"),
                memory: f(&limits.map(|l| l.memory.as_ref()), "2Gi"),
            },
        })
        .render()
    }
}
