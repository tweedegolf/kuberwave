use crate::cluster::templates::*;
use crate::cluster::types::*;

fn parse_quick<T: serde::de::DeserializeOwned>(path: &std::path::Path) -> T {
    let f = std::fs::File::open(path).unwrap();
    serde_yaml::from_reader(f).unwrap()
}

pub fn exec(path: &std::path::Path, destination: &std::path::Path) {
    let manifest: ClusterManifest = parse_quick(path);
    let destination_base = destination.to_owned();

    let mut apply_files = vec![];
    for namespace in manifest.namespaces {
        apply_files.push(NamespaceTemplate::instantiate(&namespace));
    }

    let mut auth_files = vec![];
    for user in manifest.users {
        if user.kind == UserKind::ServiceAccount {
            apply_files.push(ServiceAccountTemplate::instantiate(&user));
        }

        if let Some(cluster_permissions) = &user.cluster_permissions {
            for cluster_permission in cluster_permissions {
                auth_files.push(ClusterRolebindingTemplate::instantiate(
                    cluster_permission,
                    &user.name,
                    user.kind,
                ));
            }
        }

        if let Some(ns_perms_binding) = &user.permissions {
            for (namespace, permissions) in ns_perms_binding {
                for permission in permissions {
                    auth_files.push(RolebindingTemplate::instantiate(
                        permission, namespace, &user.name, user.kind,
                    ));
                }
            }
        }
    }

    let destination_apply = destination_base.join("apply");
    for file in &apply_files {
        file.write(destination_apply.clone()).unwrap();
    }

    let destination_auth = destination_base.join("auth");
    for file in &auth_files {
        file.write(destination_auth.clone()).unwrap();
    }

    println!(
        "{} files generated in {}.",
        apply_files.len() + auth_files.len(),
        destination.to_string_lossy()
    );
}
