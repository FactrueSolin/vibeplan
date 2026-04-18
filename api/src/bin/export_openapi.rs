fn main() -> anyhow::Result<()> {
    let document = plan_api::openapi::openapi_json_value();
    let output = serde_json::to_string_pretty(&document)?;
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/openapi.json");
    std::fs::write(path, output)?;
    Ok(())
}
