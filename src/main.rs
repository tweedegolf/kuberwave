pub mod cluster;
pub mod commands;
pub mod error;
pub mod project;
pub mod resourcefile;

mod kubectl;
mod secrets;
mod util;

use crate::commands::*;
use crate::error::Result;

fn run() -> Result<()> {
    let manifest_arg = clap::Arg::with_name("manifest")
        .required(true)
        .value_name("manifest-path")
        .takes_value(true)
        .help("Path to manifest file");

    let output_dir_arg = clap::Arg::with_name("output")
        .short("o")
        .long("output")
        .default_value("./result")
        .value_name("output-path")
        .takes_value(true)
        .help("Path to output directory");

    let inventory_arg = clap::Arg::with_name("inventory")
        .short("i")
        .long("inventory")
        .value_name("inventory-path")
        .takes_value(true)
        .help("Path to inventory file");

    let token_arg = clap::Arg::with_name("token")
        .short("t")
        .long("token")
        .value_name("token-path")
        .takes_value(true)
        .help("Path to token file, encrypted with ansible-vault");

    let token_type_arg = clap::Arg::with_name("token-type")
        .long("token-type")
        .value_name("type")
        .takes_value(true)
        .default_value("ansible-vault")
        .help("Encryption type (either sops or ansible-vault)");

    let kubeconfig_arg = clap::Arg::with_name("kubeconfig")
        .short("c")
        .long("kubeconfig")
        .value_name("kubeconfig-path")
        .takes_value(true)
        .help("Path to kubeconfig file");

    let dry_run_arg = clap::Arg::with_name("dry-run")
        .short("d")
        .long("dry-run")
        .help("Do not actually write the configuration");

    let generate_args = &[dry_run_arg, inventory_arg, manifest_arg.clone()];

    let matches = clap::App::new(clap::crate_name!())
        .about(clap::crate_description!())
        .version(clap::crate_version!())
        .subcommand(
            clap::SubCommand::with_name("cluster-generate")
                .about("Generates a cluster configuration and writes to a directory")
                .arg(&output_dir_arg)
                .arg(&manifest_arg),
        )
        .subcommand(
            clap::SubCommand::with_name("generate")
                .about("Generates a configuration and writes to a directory")
                .arg(&output_dir_arg)
                .args(generate_args),
        )
        .subcommand(
            clap::SubCommand::with_name("deploy")
                .about("Deploys a configuration to the current cluster")
                .arg(token_type_arg)
                .arg(token_arg)
                .arg(kubeconfig_arg)
                .args(generate_args),
        )
        .get_matches();

    match matches.subcommand() {
        ("cluster-generate", Some(opts)) => {
            let manifest = clap::value_t_or_exit!(opts.value_of("manifest"), std::path::PathBuf);
            let output = clap::value_t_or_exit!(opts.value_of("output"), std::path::PathBuf);

            cluster_generate::exec(&manifest, &output)
        }
        ("generate", Some(opts)) => {
            let project_opts = ProjectOpts::parse(opts);
            let output = clap::value_t_or_exit!(opts.value_of("output"), std::path::PathBuf);

            project_generate::exec(&project_opts, &output)?
        }
        ("deploy", Some(opts)) => {
            let project_opts = ProjectOpts::parse(opts);
            let kubeconfig = clap::value_t!(opts.value_of("kubeconfig"), std::path::PathBuf).ok();
            let token = clap::value_t!(opts.value_of("token"), std::path::PathBuf);
            let token_type = clap::value_t!(
                opts.value_of("token-type"),
                crate::project::types::EncryptionType
            );
            let token_source = match (token_type, token) {
                (Ok(e), Ok(token)) => Some((e, token)),
                _ => None,
            };

            project_deploy::exec(project_opts, kubeconfig, token_source)?
        }
        _ => (),
    }

    Ok(())
}

fn main() {
    ::std::process::exit(match run() {
        Ok(_) => 0,
        Err(err) => {
            use failure::AsFail;
            eprintln!("{}", err);

            for cause in err.as_fail().iter_causes() {
                eprintln!("cause: {}", cause);
            }

            1
        }
    });
}
