const URL: &str = "https://rpc-lb.neutron.org:443";

async fn get_latest_block_height(url: &str) -> anyhow::Result<u64> {
    let command = format!("{}/status", url);
    let response: serde_json::Value = reqwest::get(command).await?.json().await?;
    let height = &response["result"]["sync_info"]["latest_block_height"];
    Ok(height.as_str().unwrap().parse::<u64>()?)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let height = get_latest_block_height(URL).await?;
    println!("block height = {}", height);
    Ok(())
}
