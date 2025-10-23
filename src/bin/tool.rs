use loco_rs::cli;
use myapp::app::App;

#[allow(clippy::result_large_err)]
#[tokio::main]
async fn main() -> loco_rs::Result<()> {
    cli::main::<App>().await
}
