use std::env;
use std::path::PathBuf;

fn main () -> Result<(), Box<dyn std::error::Error>> {

    let target_dir = PathBuf::from(env::var("OUT_DIR")?);
    tonic_build::configure()
        .file_descriptor_set_path(target_dir.join("skunkr.bin"))// proto package name
        .compile(&["proto/api.proto"], &["proto"])?;

    Ok(())
}