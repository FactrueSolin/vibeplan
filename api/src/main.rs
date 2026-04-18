#[tokio::main]
async fn main() -> anyhow::Result<()> {
    plan_api::run().await
}
