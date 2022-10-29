use dprint_core::plugins::process;
use plugin::ClangPluginHandler;

mod configuration;
mod plugin;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if let Some(parent_process_id) = process::get_parent_process_id_from_cli_args() {
        process::start_parent_process_checker_task(parent_process_id);
    }

    process::handle_process_stdio_messages(ClangPluginHandler).await
}
