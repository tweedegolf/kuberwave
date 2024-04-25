use crate::error::Result;
use crate::project::types::EncryptionType;
use failure::ResultExt;
use std::path::Path;

mod sops {
    use crate::error::Result;
    use failure::ResultExt;
    use std::ffi::OsStr;
    use std::path::Path;
    use std::process::Command;

    pub fn read_from_file(file: &Path, age_keys_env: Option<&str>) -> Result<Vec<u8>> {
        let args = [OsStr::new("--decrypt"), file.as_os_str()];
        let mut command = Command::new("sops");
        command.args(args);
        if let Some(k) = age_keys_env {
            command.env("SOPS_AGE_KEY", k);
        }
        let output = command
            .output()
            .context(crate::error::ErrorKind::SOPSError {
                file: file.to_owned(),
            })?;

        if !output.status.success() {
            let message = std::str::from_utf8(&output.stderr)
                .unwrap_or_default()
                .to_owned();
            return Err(crate::error::Error::create(
                message,
                crate::error::ErrorKind::SOPSError {
                    file: file.to_owned(),
                },
            ));
        }
        Ok(output.stdout)
    }
}

pub fn read_secret_from_file(
    encryption: EncryptionType,
    file: &Path,
    vault_key: Option<&str>,
) -> Result<Vec<u8>> {
    match (encryption, vault_key) {
        (EncryptionType::AnsibleVault, None) => Err(crate::error::ErrorKind::AnsibleVaultError {
            file: file.to_owned(),
        }
        .into()),
        (EncryptionType::AnsibleVault, Some(vault_key)) => Ok(
            ansible_vault::decrypt_vault_from_file(file, vault_key).context(
                crate::error::ErrorKind::AnsibleVaultError {
                    file: file.to_owned(),
                },
            )?,
        ),
        (EncryptionType::SOPS, k) => sops::read_from_file(file, k),
    }
}
