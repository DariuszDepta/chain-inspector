const URL: &str = "https://rpc-lb.neutron.org:443";

/// Gets the height of the latest block.
///
/// Equivalent of:
///
/// ```shell
/// curl -s "https://rpc-lb.neutron.org:443/status" | jq -r '.result.sync_info.latest_block_height'
/// ```
async fn get_latest_block_height(url: &str) -> anyhow::Result<u64> {
    let command = format!("{}/status", url);
    let response: serde_json::Value = reqwest::get(command).await?.json().await?;
    let height = &response["result"]["sync_info"]["latest_block_height"];
    Ok(height.as_str().unwrap().parse::<u64>()?)
}

/// Retrieves transaction hashes.
///
/// Equivalent of:
///
/// ```shell
/// curl -s "https://rpc-lb.neutron.org:443/tx_search?query=\"tx.height=51923449\"" | jq '[.result.txs[].hash]'
/// ```
async fn get_transaction_hashes(url: &str, height: u64) -> anyhow::Result<Vec<String>> {
    let command = format!(r#"{}/tx_search?query="tx.height={}""#, url, height);
    let response: serde_json::Value = reqwest::get(command).await?.json().await?;
    let transactions = &response["result"]["txs"].as_array();
    let mut hashes = vec![];
    for tx in transactions.unwrap().iter().skip(1) {
        hashes.push(tx["hash"].as_str().unwrap().to_string());
    }
    Ok(hashes)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut height = get_latest_block_height(URL).await?;
    println!("last block height = {}", height);
    let mut count = 100;
    while count > 0 {
        let hashes = get_transaction_hashes(URL, height).await?;
        if !hashes.is_empty() {
            println!("height = {}\n  {:?}", height, hashes);
        }
        height -= 1;
        count -= 1;
    }
    Ok(())
}
