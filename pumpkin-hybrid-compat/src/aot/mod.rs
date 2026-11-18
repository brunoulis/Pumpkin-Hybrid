mod bytecode;
mod native;

use crate::Result;
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File},
    io::{self, Read, Seek},
    path::{Path, PathBuf},
    process::Command,
};

#[cfg(test)]
mod tests;

pub struct Execution {
    pub value: Option<[i32; 3]>,
    pub module: PathBuf,
    pub cache_hit: bool,
}

pub fn execute(jar: &Path, cache: &Path, input: Option<[f64; 3]>) -> Result<Execution> {
    let mut archive = File::open(jar)?;
    let jar_hash = digest(&mut archive)?;
    archive.rewind()?;
    let class = bytecode::read_class(archive)?;
    let target = native::target()?;

    let mut key = Sha256::new();
    key.update(include_bytes!("../../Cargo.lock"));
    key.update(include_bytes!("bytecode.rs"));
    key.update(include_bytes!("native.rs"));
    key.update(include_bytes!("mod.rs"));
    key.update(jar_hash.as_bytes());
    key.update(Sha256::digest(&class));
    key.update(target.triple().to_string());
    for flag in target.isa_flags() {
        key.update(flag.to_string());
    }
    let key = format!("{:x}", key.finalize());

    fs::create_dir_all(cache)?;
    let directory = cache.canonicalize()?.join(&key);
    fs::create_dir_all(&directory)?;
    let lock = File::options()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(directory.join(".lock"))?;
    lock.lock()?;

    let module = directory.join("module.so");
    let manifest = directory.join("manifest");
    let cache_hit = has_valid_artifact(&module, &manifest, &key)?;
    if !cache_hit {
        let function = bytecode::translate(&class)?;
        let generated = native::emit(&function, target)?;
        let staging = tempfile::tempdir_in(&directory)?;
        let object = staging.path().join("module.o");
        let library = staging.path().join("module.so");
        fs::write(&object, generated)?;
        let status = Command::new("cc")
            .args(["-shared", "-Wl,-z,defs", "-o"])
            .arg(&library)
            .arg(&object)
            .arg("-lm")
            .status()?;
        if !status.success() {
            return Err(format!("No se pudo enlazar el módulo AOT ({status})").into());
        }
        let hash = digest(File::open(&library)?)?;
        let declaration = staging.path().join("manifest");
        fs::write(&declaration, manifest_text(&key, &hash))?;
        fs::rename(library, &module)?;
        fs::rename(declaration, &manifest)?;
    }

    let value = native::invoke(&module, input)?;
    Ok(Execution {
        value,
        module,
        cache_hit,
    })
}

fn has_valid_artifact(library: &Path, manifest: &Path, key: &str) -> Result<bool> {
    let files = fs::read_to_string(manifest).and_then(|data| Ok((data, File::open(library)?)));
    match files {
        Ok((data, artifact)) => Ok(data == manifest_text(key, &digest(artifact)?)),
        Err(error)
            if matches!(
                error.kind(),
                io::ErrorKind::NotFound | io::ErrorKind::InvalidData
            ) =>
        {
            Ok(false)
        }
        Err(error) => Err(error.into()),
    }
}

fn manifest_text(key: &str, module_hash: &str) -> String {
    format!(
        "abi={}\nkey={key}\nsha256={module_hash}\n",
        native::ABI_VERSION
    )
}

fn digest(mut source: impl Read) -> io::Result<String> {
    let mut hash = Sha256::new();
    let mut buffer = [0; 32 * 1024];
    loop {
        let size = source.read(&mut buffer)?;
        if size == 0 {
            break;
        }
        hash.update(&buffer[..size]);
    }
    Ok(format!("{:x}", hash.finalize()))
}

pub fn print_execution(jar: &Path, cache: &Path, input: Option<[f64; 3]>) -> Result<()> {
    let run = execute(jar, cache, input)?;
    match run.value {
        Some([x, y, z]) => println!("{x},{y},{z}"),
        None => println!("NullPointerException"),
    }
    let cache = if run.cache_hit {
        "reutilizado"
    } else {
        "compilado"
    };
    eprintln!("Módulo AOT {cache} en {}", run.module.display());
    Ok(())
}
