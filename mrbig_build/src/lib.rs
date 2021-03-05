use std::io::{Result, Write};
use std::path::{Path, PathBuf};

mod protos;

/// Compile the proto files to generate the necessary code
/// for gRPC servers and reflection server.
///
/// This method uses other compile methods from
/// `grpc_reflection_build` crate and `tonic_build`,
/// which depend on `prost_build`.
///
/// This method relies on OUT_DIR being set. It fails otherwise.
pub fn compile_protos<P>(protos: &[P], includes: &[P]) -> Result<()>
where
    P: AsRef<Path>,
{
    let tmp = tempfile::Builder::new().prefix("mrbig-build").tempdir()?;
    let builtin_include = tmp.path().join("mrbig-builtin-proto");

    std::fs::create_dir(&builtin_include)?;

    let mut builtin_protos: Vec<PathBuf> = vec![];
    #[cfg(feature = "grpc")]
    {
        for (filename, contents) in &[
            ("health.proto", protos::HEALTH),
            #[cfg(feature = "reflection")]
            ("reflection.proto", protos::REFLECTION),
        ] {
            let path = builtin_include.join(filename);
            let mut file = std::fs::File::create(&path)?;
            file.write_all(contents.as_bytes())?;
            builtin_protos.push(path);
        }
    }

    let protos: Vec<&str> = protos
        .iter()
        .map(|p| (*p).as_ref())
        .chain(builtin_protos.iter().map(|p| p.as_path()))
        .map(path_to_str)
        .collect();

    let includes: Vec<&str> = includes
        .iter()
        .map(|p| (*p).as_ref())
        .chain([&builtin_include].iter().map(|p| p.as_path()))
        .map(path_to_str)
        .collect();

    #[cfg(feature = "grpc")]
    {
        #[cfg(feature = "reflection")]
        {
            grpc_reflection_build::compile_protos(&protos, &includes)?;
        }

        // tonic_build runs rustfmt in OUT_DIR
        // but we shouldn't count on it(!) @TODO
        tonic_build::configure().compile(&protos, &includes)?;
    }

    Ok(())
}

fn path_to_str(path: &Path) -> &str {
    path.to_str()
        .expect("paths must be valid unicode sequences")
}
