use super::{Artifact, Result};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

const EXTENSIONS: &str = include_str!("../teleport/artifacts.lock");

pub fn prepare(instance: &Path) -> Result<()> {
    let instance = instance.canonicalize()?;
    let base = instance.join("plugins/data/patchbukkit");
    let patchbukkit_jar = base.join("jassets/patchbukkit.jar");
    let paper = base.join("cache/paper");
    let paper_jar = paper.join("versions/26.3/paper-26.3.jar");
    if !patchbukkit_jar.is_file() || !paper_jar.is_file() {
        return Err(
            "Arranca y detén PatchBukkit con Paper 26.3 antes de preparar el piloto".into(),
        );
    }

    let plugins = base.join("patchbukkit-plugins");
    let dependencies = plugins.join("PumpkinHybridTeleport/jars");
    for line in EXTENSIONS.lines().filter(|line| !line.starts_with('#')) {
        Artifact::parse(line)?.install(&dependencies)?;
    }

    let mut jars = vec![patchbukkit_jar, paper_jar];
    collect_jars(&paper.join("libraries"), &mut jars)?;
    let work = base.join(format!(".teleport-build-{}", std::process::id()));
    fs::create_dir(&work)?;
    let built = build(&work, &jars).and_then(|()| {
        fs::rename(
            work.join("teleport-hook.jar"),
            instance.join("teleport-hook.jar"),
        )?;
        fs::rename(
            work.join("teleport-bridge.jar"),
            plugins.join("teleport-bridge.jar"),
        )?;
        Ok(())
    });
    let clean = fs::remove_dir_all(work);
    built?;
    clean?;
    println!(
        "Agente preparado. Añade -javaagent:teleport-hook.jar a [jvm].extra-args de PatchBukkit."
    );
    Ok(())
}

fn build(work: &Path, jars: &[PathBuf]) -> Result<()> {
    let sources = [
        (
            "TeleportHook.java",
            include_bytes!("../teleport/TeleportHook.java").as_slice(),
        ),
        (
            "TeleportBridge.java",
            include_bytes!("../teleport/TeleportBridge.java").as_slice(),
        ),
        (
            "MANIFEST.MF",
            include_bytes!("../teleport/MANIFEST.MF").as_slice(),
        ),
    ];
    for (name, contents) in sources {
        fs::write(work.join(name), contents)?;
    }

    fs::create_dir(work.join("agent"))?;
    fs::create_dir(work.join("plugin"))?;
    let java_home = env::var_os("JAVA_HOME").map(fs::canonicalize).transpose()?;
    let java_tool = |name: &str| {
        let binary = java_home
            .as_ref()
            .map_or_else(|| PathBuf::from(name), |home| home.join("bin").join(name));
        let mut command = Command::new(binary);
        command.current_dir(work);
        command
    };
    let classpath = env::join_paths(jars)?;
    run(java_tool("javac")
        .args(["--release", "25", "-cp"])
        .arg(classpath)
        .args(["-d", "agent", "TeleportHook.java"]))?;
    run(java_tool("jar").args([
        "--create",
        "--file",
        "teleport-hook.jar",
        "--manifest",
        "MANIFEST.MF",
        "-C",
        "agent",
        ".",
    ]))?;

    let classpath = env::join_paths(
        jars.iter()
            .map(PathBuf::as_path)
            .chain([Path::new("agent")]),
    )?;
    run(java_tool("javac")
        .args(["--release", "25", "-cp"])
        .arg(classpath)
        .args(["-d", "plugin", "TeleportBridge.java"]))?;
    fs::write(
        work.join("plugin/plugin.yml"),
        include_bytes!("../teleport/plugin.yml"),
    )?;
    run(java_tool("jar").args([
        "--create",
        "--file",
        "teleport-bridge.jar",
        "-C",
        "plugin",
        ".",
    ]))?;

    Ok(())
}

fn collect_jars(directory: &Path, jars: &mut Vec<PathBuf>) -> std::io::Result<()> {
    let mut entries = fs::read_dir(directory)?.collect::<std::io::Result<Vec<_>>>()?;
    entries.sort_by_key(fs::DirEntry::file_name);
    for entry in entries {
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            collect_jars(&path, jars)?;
        } else if path.extension().is_some_and(|extension| extension == "jar") {
            jars.push(path);
        }
    }
    Ok(())
}

fn run(command: &mut Command) -> Result<()> {
    let exit = command.status()?;
    if !exit.success() {
        return Err(format!("{:?} terminó con {exit}", command.get_program()).into());
    }
    Ok(())
}
