pub mod cluster_generate;
pub mod project_deploy;
pub mod project_generate;

use crate::error::{ErrorKind, Result};
use crate::kubectl::Kubectl;
use crate::project::types::Manifest;
use crate::resourcefile::Resourcefile;
use failure::ResultExt;

#[derive(Debug)]
pub struct ProjectOpts {
    manifest: std::path::PathBuf,
    inventory: Option<std::path::PathBuf>,
    dry_run: bool,
}

impl ProjectOpts {
    pub fn parse(opts: &clap::ArgMatches) -> ProjectOpts {
        ProjectOpts {
            manifest: clap::value_t_or_exit!(opts.value_of("manifest"), std::path::PathBuf),
            inventory: clap::value_t!(opts.value_of("inventory"), std::path::PathBuf).ok(),
            dry_run: opts.is_present("dry-run"),
        }
    }
}

fn load_inventory(path: &std::path::Path) -> Result<Vec<yaml_rust::Yaml>> {
    let content = std::fs::read_to_string(path).context(ErrorKind::FileReadError {
        name: path.to_owned(),
    })?;
    let yaml = yaml_rust::YamlLoader::load_from_str(&content).context(ErrorKind::ParseError {
        name: path.to_owned(),
    })?;

    Ok(yaml)
}

fn load_manifest_with_context(path: &std::path::Path, context: &tera::Context) -> Result<Manifest> {
    let content: std::string::String = crate::util::process_template(path, context)?;

    let manifest: Manifest = serde_yaml::from_str(&content).context(ErrorKind::ParseError {
        name: path.to_owned(),
    })?;

    Ok(manifest)
}

pub type LoadedManifest<'a> = (Manifest, tera::Context);

pub fn load_manifest<'a>(opts: &ProjectOpts) -> Result<LoadedManifest<'a>> {
    let context = match &opts.inventory {
        Some(inventory) => {
            let inventory = load_inventory(inventory).context(ErrorKind::InventoryError)?;
            crate::util::map_yaml_to_context(inventory).context(ErrorKind::InventoryError)?
        }
        None => tera::Context::new(),
    };

    let manifest =
        load_manifest_with_context(&opts.manifest, &context).context(ErrorKind::ManifestError)?;

    Ok((manifest, context))
}

pub fn compute_project_files(
    opts: &ProjectOpts,
    (manifest, context): &LoadedManifest<'_>,
    kubectl: Option<&Kubectl>,
) -> Result<Vec<Resourcefile>> {
    let base = opts.manifest.parent().unwrap().to_owned();
    let secret = crate::util::get_secret();
    let secret = secret.as_deref();

    let files = manifest
        .to_sourcefiles(&base, context.clone(), secret, kubectl)
        .context(ErrorKind::ManifestError)?;

    Ok(files)
}
