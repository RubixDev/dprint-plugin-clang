use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Result, bail};
use serde_json::{json, Map, Value};

use crate::{project_root, WORKSPACE_CONFIG};

const TARGETS: &[(&str, &str)] = &[
    ("linux-x86_64", "x86_64-unknown-linux-musl"),
    ("windows-x86_64", "x86_64-pc-windows-gnu"),
];

pub fn main() -> Result<()> {
    let bin = &WORKSPACE_CONFIG.package.name;
    let version = &WORKSPACE_CONFIG.package.version;
    let repo = &WORKSPACE_CONFIG.package.repository;
    let dest_dir = project_root().join("package");

    fs::remove_dir_all(&dest_dir).unwrap_or(());

    let mut plugin_json = Map::new();
    plugin_json.insert("schemaVersion".into(), 2.into());
    plugin_json.insert("kind".into(), "process".into());
    plugin_json.insert("name".into(), bin.as_str().into());
    plugin_json.insert("version".into(), version.as_str().into());

    for (platform, target) in TARGETS {
        eprintln!("Compiling for target {target}...");

        let status = Command::new(env!("CARGO"))
            .current_dir(project_root())
            .args(["build", "--release", "--target", target])
            .status()?;
        if !status.success() {
            bail!("cargo build for target {target} failed");
        }

        let build_dir = build_dir(target);
        fs::create_dir_all(&dest_dir).unwrap_or(());

        let bin_name = match target.contains("windows") {
            true => format!("{bin}.exe"),
            false => bin.to_owned(),
        };
        fs::copy(build_dir.join(&bin_name), dest_dir.join(&bin_name))?;

        let zip_name = format!("{bin}-{target}.zip");
        Command::new("zip")
            .current_dir(&dest_dir)
            .arg(&zip_name)
            .arg(&bin_name)
            .spawn()?
            .wait()?;
        fs::remove_file(dest_dir.join(bin_name))?;

        plugin_json.insert(
            platform.to_string(),
            json!({
                "reference": format!("{repo}/releases/download/{version}/{zip_name}"),
                "checksum": sha256::try_digest(&*dest_dir.join(zip_name))?,
            }),
        );
    }

    fs::write(
        dest_dir.join("plugin.json"),
        format!("{:#}", Value::Object(plugin_json)),
    )?;
    fs::write(
        dest_dir.join("release.json"),
        format!(
            "{:#}",
            json!({
                "clang": {
                    "BasedOnStyle": "Mozilla"
                },
                "includes": ["**/*.{cs,java,mjs,js,ts,json,m,mm,proto,protodevel,td,textpb,pb.txt,textproto,asciipb,ssv,svh,v,vh,c,h,cc,hh,cpp,hpp,c++,C,cxx}"],
                "plugins": [
                    format!("https://plugins.dprint.dev/RubixDev/clang-{version}.json@{}", sha256::try_digest(&*dest_dir.join("plugin.json"))?)
                ]
            })
        ),
    )?;

    Command::new("exa")
        .arg("-lahg")
        .arg("--tree")
        .arg(&dest_dir)
        .spawn()?;
    Ok(())
}

fn build_dir(target: &str) -> PathBuf {
    match env::var("CARGO_TARGET_DIR") {
        Ok(dir) => Path::new(&dir).join(target).join("release"),
        Err(_) => project_root().join("target").join(target).join("release"),
    }
}
