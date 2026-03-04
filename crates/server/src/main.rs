#![deny(unsafe_code)]

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("agentbin-server starting");
    Ok(())
}
