use std::env;

use crate::commands::{compute_project_files, load_manifest, ProjectOpts};
use crate::error::{ErrorKind, Result};
use crate::project::types::EncryptionType;
use crate::util;

pub(crate) const DEPLOY_TOKEN_NAME: &str = "DEPLOY_TOKEN"; 

pub fn exec(
    opts: ProjectOpts,
    kubeconfig: Option<std::path::PathBuf>,
    token_source: Option<(EncryptionType, std::path::PathBuf)>,
) -> Result<()> {
    use failure::ResultExt;

    let secret = util::get_secret();
    let secret = secret.as_deref();

    let token: Option<String> = match token_source {
        Some((encryption, path)) => Some(
            String::from_utf8(
                crate::secrets::read_secret_from_file(encryption, &path, secret)
                    .context(ErrorKind::TokenError)?,
            )
            .context(ErrorKind::TokenError)?,
        ),
        None => {
            if let Some(token) = env::var(DEPLOY_TOKEN_NAME).ok() {
                Some(token)
            } else {
                None
            }
        }
    };

    let loaded_manifest = load_manifest(&opts)?;
    let (manifest, _) = &loaded_manifest;

    let kubectl = crate::kubectl::Kubectl::new(kubeconfig, token, Some(manifest.namespace.clone()));

    let files = compute_project_files(&opts, &loaded_manifest, Some(&kubectl))?;

    if opts.dry_run {
        println!("{} files generated (dry-run).", files.len());
    } else {
        for file in &files {
            kubectl.apply(file)?;
        }

        println!("{} files deployed.", files.len());
    }

    Ok(())
}
