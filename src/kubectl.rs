use crate::error::{Error, ErrorKind};
use crate::resourcefile::Resourcefile;
use std::process::Command;

pub struct Kubectl {
    kubeconfig: Option<std::path::PathBuf>,
    token: Option<String>,
    namespace: Option<String>,
}

impl Kubectl {
    pub fn new(
        kubeconfig: Option<std::path::PathBuf>,
        token: Option<String>,
        namespace: Option<String>,
    ) -> Kubectl {
        Kubectl {
            kubeconfig,
            token,
            namespace,
        }
    }

    fn load_default_args(&self, cmd: &mut Command) {
        match &self.namespace {
            Some(n) => {
                cmd.arg("-n").arg(n.trim());
            }
            None => (),
        }

        match &self.kubeconfig {
            Some(kc) => {
                cmd.arg("--kubeconfig").arg(kc);
            }
            None => (),
        }

        match &self.token {
            Some(t) => {
                cmd.arg("--token").arg(t.trim());
            }
            None => (),
        }
    }

    pub fn get_resource_version(
        &self,
        resource_type: &str,
        name: &str,
    ) -> crate::error::Result<Option<String>> {
        use failure::ResultExt;
        let mut cmd = Command::new("kubectl");

        cmd.arg("get").arg("-ojsonpath={.metadata.resourceVersion}");

        self.load_default_args(&mut cmd);

        cmd.arg(resource_type);
        cmd.arg(name);

        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        let output = cmd.output().context(ErrorKind::KubectlError)?;
        let status = output.status;

        if !status.success() {
            let error = String::from_utf8(output.stderr).context(ErrorKind::KubectlError)?;
            if error.trim().ends_with("not found") {
                return Ok(None);
            } else {
                return Err(Error::create(
                    format!(
                        "Unexpected kubectl exec code {} for {}\n{}",
                        status.code().unwrap(),
                        name,
                        error,
                    ),
                    ErrorKind::KubectlError,
                ));
            }
        }

        let resource_version: String =
            String::from_utf8(output.stdout).context(ErrorKind::KubectlError)?;

        Ok(Some(resource_version))
    }

    pub fn apply(&self, file: &Resourcefile) -> crate::error::Result<()> {
        use failure::ResultExt;

        let mut cmd = Command::new("kubectl");

        cmd.arg("apply").arg("-f").arg("-");

        self.load_default_args(&mut cmd);

        cmd.stdin(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        let mut proc = cmd.spawn().context(ErrorKind::KubectlError)?;

        use std::io::Write;
        proc.stdin
            .as_mut()
            .unwrap()
            .write_all(file.buffer.as_bytes())
            .context(ErrorKind::KubectlError)?;

        let output = proc.wait_with_output().context(ErrorKind::KubectlError)?;

        if !output.status.success() {
            return Err(Error::create(
                format!(
                    "Unexpected kubectl exec code {} for {}\n{}",
                    output.status.code().unwrap(),
                    file.name,
                    String::from_utf8(output.stderr).unwrap(),
                ),
                ErrorKind::KubectlError,
            ));
        }

        Ok(())
    }
}
