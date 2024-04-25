use crate::commands::{compute_project_files, load_manifest, ProjectOpts};
use crate::error::Result;

pub fn exec(opts: &ProjectOpts, output: &std::path::Path) -> Result<()> {
    let loaded_manifest = load_manifest(opts)?;

    // Assume we do not have access to kubectl in the generate setting; potentially breaking the
    // generate and kubectl apply -f scenario.
    let files = compute_project_files(opts, &loaded_manifest, None)?;

    if opts.dry_run {
        println!("{} files generated (dry-run).", files.len());
    } else {
        for file in &files {
            file.write(output.to_path_buf()).unwrap();
        }

        println!(
            "{} files generated in {}.",
            files.len(),
            output.to_string_lossy()
        );
    }

    Ok(())
}
