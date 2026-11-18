use std::{
    env,
    error::Error,
    fs::{self, File},
    io::{self, Write},
    path::Path,
    process::{Command, ExitCode, Stdio},
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

const PROFILE: &str = include_str!("../artifacts.lock");
const PROPERTIES: &str = "server-ip=127.0.0.1\nserver-port=25575\nonline-mode=true\nenable-rcon=false\nenable-query=false\nview-distance=4\nsimulation-distance=4\nmax-players=5\nmotd=Pumpkin Hybrid Compat - NeoForge 1.21.1\n";

fn main() -> ExitCode {
    match execute() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn execute() -> Result<()> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let artifacts = PROFILE
        .lines()
        .filter(|line| !line.starts_with('#') && !line.is_empty())
        .map(Artifact::parse)
        .collect::<Result<Vec<_>>>()?;
    let instance = Path::new(".instance");
    match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["prepare"] | ["prepare", "--accept-eula"] => {
            fs::create_dir_all(instance)?;
            write_default(&instance.join("server.properties"), PROPERTIES)?;
            for artifact in &artifacts {
                artifact.install(instance)?;
            }
            if args.len() == 2 {
                fs::write(instance.join("eula.txt"), "eula=true\n")?;
            }
        }
        ["prepare-client"] => {
            for artifact in artifacts.iter().filter(|entry| entry.role == "mod") {
                artifact.install(Path::new(".client"))?;
            }
        }
        ["run"] => {
            let eula = match fs::read_to_string(instance.join("eula.txt")) {
                Ok(value) => value,
                Err(error) if error.kind() == io::ErrorKind::NotFound => String::new(),
                Err(error) => return Err(error.into()),
            };
            if !eula.lines().any(|line| line.trim() == "eula=true") {
                return Err("Lee https://www.minecraft.net/eula y, si aceptas, ejecuta prepare --accept-eula".into());
            }
            for artifact in &artifacts {
                verify(&instance.join(artifact.path), artifact.hash)?;
            }
            let server = artifacts
                .iter()
                .find(|entry| entry.role == "server")
                .ok_or("El perfil no contiene un servidor")?;
            let local_java = Path::new(".local/java/bin/java");
            let java = match env::var_os("PUMPKIN_COMPAT_JAVA") {
                Some(value) if Path::new(&value).components().count() > 1 => {
                    fs::canonicalize(value)?.into_os_string()
                }
                Some(value) => value,
                None if local_java.is_file() => local_java.canonicalize()?.into_os_string(),
                None => "java".into(),
            };
            let exit = Command::new(java)
                .current_dir(instance)
                .args([
                    "-Duser.language=en",
                    "-Duser.country=US",
                    "-Dterminal.ansi=false",
                    "-Xms512M",
                    "-Xmx4G",
                    "-jar",
                    server.path,
                    "nogui",
                ])
                .status()?;
            if !exit.success() {
                return Err(format!("Arclight terminó con {exit}").into());
            }
        }
        _ => {
            return Err(
                "Uso: pumpkin-hybrid-compat prepare [--accept-eula] | prepare-client | run".into(),
            );
        }
    }
    Ok(())
}

struct Artifact<'a> {
    role: &'a str,
    path: &'a str,
    hash: &'a str,
    url: &'a str,
}

impl<'a> Artifact<'a> {
    fn parse(line: &'a str) -> Result<Self> {
        let fields = line.split_whitespace().collect::<Vec<_>>();
        let [role, path, hash, url] = fields.as_slice() else {
            return Err(format!("Entrada incorrecta en artifacts.lock: {line}").into());
        };
        Ok(Self {
            role,
            path,
            hash,
            url,
        })
    }

    fn install(&self, directory: &Path) -> Result<()> {
        let path = directory.join(self.path);
        if path.try_exists()? {
            return verify(&path, self.hash);
        }
        fs::create_dir_all(path.parent().ok_or("Ruta de descarga sin directorio")?)?;
        let partial = path.with_extension(format!("jar.part-{}", std::process::id()));
        let result = (|| {
            let exit = Command::new("curl")
                .args([
                    "--fail",
                    "--silent",
                    "--show-error",
                    "--location",
                    "--retry",
                    "2",
                    "--max-time",
                    "180",
                    "--proto",
                    "=https",
                    "--proto-redir",
                    "=https",
                    "--output",
                ])
                .arg(&partial)
                .arg(self.url)
                .status()?;
            if !exit.success() {
                return Err(format!("Error al descargar {} ({exit})", self.url).into());
            }
            verify(&partial, self.hash)?;
            fs::rename(&partial, &path)?;
            println!("Instalado {}", path.display());
            Ok(())
        })();
        if result.is_err() {
            let _ = fs::remove_file(partial);
        }
        result
    }
}

fn write_default(path: &Path, text: &str) -> io::Result<()> {
    match File::create_new(path) {
        Ok(mut file) => file.write_all(text.as_bytes()),
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => Ok(()),
        Err(error) => Err(error),
    }
}

fn verify(path: &Path, expected: &str) -> Result<()> {
    let file = File::open(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let checksum = Command::new("sha256sum")
        .stdin(Stdio::from(file))
        .output()?;
    let output = std::str::from_utf8(&checksum.stdout)?;
    if !checksum.status.success() || output.split_whitespace().next() != Some(expected) {
        return Err(format!("SHA-256 incorrecto para {}", path.display()).into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn preparation_preserves_existing_configuration() {
        let path = workspace_path("keep-config");
        fs::write(&path, "custom=true\n").unwrap();
        write_default(&path, "custom=false\n").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "custom=true\n");
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn rejects_bytes_that_do_not_match_the_lock() {
        let path = workspace_path("checksum");
        fs::write(&path, b"abc").unwrap();
        let checksum = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
        assert!(verify(&path, checksum).is_ok());
        fs::write(&path, b"tampered").unwrap();
        assert!(verify(&path, checksum).is_err());
        fs::remove_file(path).unwrap();
    }

    fn workspace_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "pumpkin-hybrid-compat-{}-{name}",
            std::process::id()
        ))
    }
}
