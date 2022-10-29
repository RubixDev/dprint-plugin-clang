use std::{
    io::Write,
    process::{Command, Stdio},
    sync::Arc,
};

use anyhow::anyhow;
use dprint_core::{
    configuration::{ConfigKeyMap, GlobalConfiguration, ResolveConfigurationResult},
    plugins::{AsyncPluginHandler, BoxFuture, FormatRequest, FormatResult, Host, PluginInfo},
};

use crate::configuration::{self, Configuration};

pub struct ClangPluginHandler;

impl AsyncPluginHandler for ClangPluginHandler {
    type Configuration = Configuration;

    fn resolve_config(
        &self,
        config: ConfigKeyMap,
        global_config: GlobalConfiguration,
    ) -> ResolveConfigurationResult<Self::Configuration> {
        configuration::resolve_config(config, &global_config)
    }

    fn plugin_info(&self) -> PluginInfo {
        PluginInfo {
            name: env!("CARGO_PKG_NAME").to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            config_key: "clang".to_string(),
            file_extensions: [
                "cs",
                "java",
                "mjs",
                "js",
                "ts",
                "json",
                "m",
                "mm",
                "proto",
                "protodevel",
                "td",
                "textpb",
                "pb.txt",
                "textproto",
                "asciipb",
                "ssv",
                "svh",
                "v",
                "vh",
                "c",
                "h",
                "cc",
                "hh",
                "cpp",
                "hpp",
                "c++",
                "C",
                "cxx",
            ]
            .into_iter()
            .map(|ext| ext.to_string())
            .collect(),
            file_names: vec![],
            help_url: concat!(env!("CARGO_PKG_REPOSITORY"), "#readme").to_string(),
            config_schema_url: "".to_string(),
            update_url: Some(
                "https://plugins.dprint.dev/RubixDev/dprint-plugin-clang/latest.json".to_string(),
            ),
        }
    }

    fn license_text(&self) -> String {
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/LICENSE")).into()
    }

    fn format(
        &self,
        request: FormatRequest<Self::Configuration>,
        _host: Arc<dyn Host>,
    ) -> BoxFuture<FormatResult> {
        Box::pin(async move {
            // range formatting is not supported
            if request.range.is_some() {
                return Ok(None);
            }

            let mut child = Command::new("clang-format")
                .arg(request.config.to_cli_flag(&request.file_text))
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()?;
            let mut stdin = child.stdin.take().unwrap();
            write!(stdin, "{}", request.file_text)?;
            drop(stdin);

            let output = child.wait_with_output()?;
            if !output.status.success() {
                return Err(anyhow!(String::from_utf8_lossy(&output.stderr).into_owned()));
            }
            let new_text = String::from_utf8_lossy(&output.stdout);

            if new_text == request.file_text {
                Ok(None)
            } else {
                Ok(Some(new_text.into_owned()))
            }
        })
    }
}
