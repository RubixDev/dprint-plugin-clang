use std::process::Stdio;

use anyhow::anyhow;
use dprint_core::{
    async_runtime::{async_trait, LocalBoxFuture},
    configuration::{ConfigKeyMap, GlobalConfiguration},
    plugins::{
        AsyncPluginHandler, FormatRequest, FormatResult, HostFormatRequest, PluginInfo,
        PluginResolveConfigurationResult,
    },
};
use tokio::{io::AsyncWriteExt, process::Command};

use crate::configuration::{self, Configuration};

pub struct ClangPluginHandler;

#[async_trait(?Send)]
impl AsyncPluginHandler for ClangPluginHandler {
    type Configuration = Configuration;

    async fn resolve_config(
        &self,
        config: ConfigKeyMap,
        global_config: GlobalConfiguration,
    ) -> PluginResolveConfigurationResult<Self::Configuration> {
        configuration::resolve_config(config, &global_config)
    }

    fn plugin_info(&self) -> PluginInfo {
        PluginInfo {
            name: env!("CARGO_PKG_NAME").to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            config_key: "clang".to_string(),
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

    async fn format(
        &self,
        request: FormatRequest<Self::Configuration>,
        _format_with_host: impl FnMut(HostFormatRequest) -> LocalBoxFuture<'static, FormatResult>
            + 'static,
    ) -> FormatResult {
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
        stdin.write_all(request.file_text.as_bytes()).await?;
        drop(stdin);

        let output = child.wait_with_output().await?;
        if !output.status.success() {
            return Err(anyhow!(String::from_utf8_lossy(&output.stderr).into_owned()));
        }
        let new_text = String::from_utf8_lossy(&output.stdout);

        if new_text == request.file_text {
            Ok(None)
        } else {
            Ok(Some(new_text.into_owned()))
        }
    }
}
