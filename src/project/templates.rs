use askama::Template;

use crate::project::types::*;
use crate::resourcefile::{Resourcefile, Resourceproto};
use std::collections::HashMap;

#[derive(Clone, Copy)]
pub struct OptRevisionVersion<'a>(pub Option<&'a str>);

impl<'a> OptRevisionVersion<'a> {
    pub fn is_some(&self) -> bool {
        self.0.is_some()
    }
    pub fn unwrap(self) -> &'a str {
        self.0.unwrap()
    }
}

pub enum IssuerMode {
    Production,
    Staging,
}

#[derive(Template)]
#[template(path = "project/certificate.yml")]
pub struct CertificateTemplate<'a> {
    resource_version: OptRevisionVersion<'a>,
    name: &'a str,
    issuer: &'a str,
    hostnames: &'a Vec<String>,
}

impl<'a> CertificateTemplate<'a> {
    /// Issue the certificate for an ingress.
    ///
    /// Currently we also support the `dns-01` ACME solver, hence the tight coupling between Ingress and Certificates is
    /// no longer conceptually required. For practical reasons we however still require it in Kuberwave.
    ///
    /// If you require a Certificate without necessarilly having an Ingress, please consider decoupling Ingresses and Certificates.
    pub fn instantiate(
        resource_version: OptRevisionVersion<'a>,
        ingress: &'a Ingress,
        mode: IssuerMode,
    ) -> Resourcefile {
        (Resourceproto {
            name: &format!("certificate-{}.yml", &ingress.certificate),
            prototype: CertificateTemplate {
                resource_version,
                name: &ingress.certificate,
                issuer: match (mode, &ingress.certificate_solver) {
                    (IssuerMode::Production, CertificateSolver::HTTP) => "kikundi-production-http",
                    (IssuerMode::Production, CertificateSolver::DNS) => "kikundi-production-dns",
                    (IssuerMode::Staging, CertificateSolver::HTTP) => "kikundi-staging-http",
                    (IssuerMode::Staging, CertificateSolver::DNS) => "kikundi-staging-dns",
                    (IssuerMode::Production, CertificateSolver::NONE) => {
                        panic!("NONE certificate resolver should not be created")
                    }
                    (IssuerMode::Staging, CertificateSolver::NONE) => {
                        panic!("NONE certificate resolver should not be created")
                    }
                },
                hostnames: &ingress.hosts,
            },
        })
        .render()
    }
}

pub struct IngressService<'a> {
    name: &'a str,
    port: u16,
}

pub struct IngressHost<'a> {
    name: &'a str,
    service: IngressService<'a>,
}

pub struct Annotation<'a> {
    key: &'a str,
    value: &'a str,
}

#[derive(Template)]
#[template(path = "project/ingress.yml")]
pub struct IngressTemplate<'a> {
    name: &'a str,
    certificate: &'a str,
    hosts: Vec<IngressHost<'a>>,
    annotations: Vec<Annotation<'a>>,
}

impl<'a> IngressTemplate<'a> {
    pub fn instantiate(ingress: &'a Ingress) -> Resourcefile {
        (Resourceproto {
            name: &format!("ingress-{}.yml", ingress.name),
            prototype: IngressTemplate {
                name: &ingress.name,
                certificate: &ingress.certificate,
                hosts: ingress
                    .hosts
                    .iter()
                    .map(|host| IngressHost {
                        name: host,
                        service: IngressService {
                            name: &ingress.service,
                            port: ingress.port,
                        },
                    })
                    .collect(),
                annotations: ingress
                    .annotations
                    .as_ref()
                    .map(|map| {
                        map.iter()
                            .map(|(key, value)| Annotation { key, value })
                            .collect()
                    })
                    .unwrap_or_default(),
            },
        })
        .render()
    }
}

fn b64encode(input: &[u8]) -> String {
    base64::encode(input)
}

#[derive(Template)]
#[template(path = "project/secrets-regcred.yml")]
pub struct SecretRegcredTemplate<'a> {
    name: &'a str,
    content: &'a str,
}

impl<'a> SecretRegcredTemplate<'a> {
    pub fn instantiate(name: &str, content: &[u8]) -> Resourcefile {
        (Resourceproto {
            name: &format!("secret-{}.yml", name),
            prototype: SecretRegcredTemplate {
                name,
                content: &b64encode(content),
            },
        })
        .render()
    }
}

pub struct Field<'a> {
    pub name: &'a str,
    pub value: &'a str,
}

#[derive(Template)]
#[template(path = "project/secrets-opaque.yml")]
pub struct SecretOpaqueTemplate<'a> {
    name: &'a str,
    secrets: Vec<Field<'a>>,
}

impl<'a> SecretOpaqueTemplate<'a> {
    pub fn instantiate(name: &str, secrets: HashMap<String, Vec<u8>>) -> Resourcefile {
        let secrets: Vec<(String, String)> = secrets
            .into_iter()
            .map(|(k, v)| (k, b64encode(&v)))
            .collect();

        let secrets: Vec<Field> = secrets
            .iter()
            .map(|(name, value)| Field { name, value })
            .collect();

        (Resourceproto {
            name: &format!("secret-{}.yml", name),
            prototype: SecretOpaqueTemplate { name, secrets },
        })
        .render()
    }
}

pub struct BinaryField<'a> {
    pub name: &'a str,
    pub value: &'a [u8],
}

#[derive(Template)]
#[template(path = "project/configmap.yml")]
pub struct ConfigMapTemplate<'a> {
    name: &'a str,
    data: &'a [Field<'a>],
    binary_data: &'a [Field<'a>],
}

impl<'a> ConfigMapTemplate<'a> {
    pub fn instantiate(
        name: &str,
        data: Vec<(String, String)>,
        binary_data: Vec<(String, Vec<u8>)>,
    ) -> Resourcefile {
        let data: Vec<Field> = data
            .iter()
            .map(|(name, value)| Field { name, value })
            .collect();

        let binary_data: Vec<(String, String)> = binary_data
            .into_iter()
            .map(|(name, value)| (name, base64::encode(value)))
            .collect();

        let binary_data: Vec<Field> = binary_data
            .iter()
            .map(|(name, value)| Field { name, value })
            .collect();

        (Resourceproto {
            name: &format!("configmap-{}.yml", name),
            prototype: ConfigMapTemplate {
                name,
                data: &data,
                binary_data: &binary_data,
            },
        })
        .render()
    }
}

#[derive(Template)]
#[template(path = "project/network-policy.yml")]
pub struct NetworkPolicyTemplate<'a> {
    namespace: &'a str,
    ingresses: &'a Vec<Ingress>,
}

impl<'a> NetworkPolicyTemplate<'a> {
    pub fn instantiate(namespace: &str, ingress: &Option<Vec<Ingress>>) -> Resourcefile {
        let empty: Vec<Ingress> = vec![];

        (Resourceproto {
            name: "network-policy.yml",
            prototype: NetworkPolicyTemplate {
                namespace,
                ingresses: match ingress {
                    Some(ingresses) => ingresses,
                    _ => &empty,
                },
            },
        })
        .render()
    }
}
