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
    let mut hashes = vec![];
    let command = format!(r#"{}/tx_search?query="tx.height={}""#, url, height);
    let response: serde_json::Value = reqwest::get(command).await?.json().await?;
    let Some(transactions) = &response["result"]["txs"].as_array() else {
        return Ok(hashes);
    };
    for tx in transactions.iter().skip(1) {
        hashes.push(tx["hash"].as_str().unwrap().to_string());
    }
    Ok(hashes)
}

/// Equivalent of:
///
/// ```shell
/// curl -s "https://rpc-lb.neutron.org:443/tx?hash=0xD542E9FC635E8D224A54B5A668BE7A3ED4FC6F09FA48F9E0ED1A5D9616295419" \
/// | jq -r '[.result.tx_result.events[] | select(.type == "message") | .attributes[] | select(.key == "action") | .value | select(startswith("/"))]'
/// ```
async fn get_msg_types(url: &str, hash: &str) -> anyhow::Result<Vec<String>> {
    let mut msg_types = vec![];
    let command = format!("{}/tx?hash=0x{}", url, hash);
    let response: serde_json::Value = reqwest::get(command).await?.json().await?;
    let Some(events) = &response["result"]["tx_result"]["events"].as_array() else {
        return Ok(msg_types);
    };
    for event in events.iter() {
        let event_type = event["type"].as_str().unwrap();
        if event_type == "message"
            && let Some(attributes) = event["attributes"].as_array()
        {
            for attribute in attributes {
                if let Some(key) = attribute["key"].as_str()
                    && key == "action"
                {
                    let value = attribute["value"].as_str().unwrap();
                    if value.starts_with("/") {
                        msg_types.push(value.to_string());
                    }
                }
            }
        }
    }
    Ok(msg_types)
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
            for hash in &hashes {
                let msg_types = get_msg_types(URL, hash).await?;
                for msg_type in msg_types {
                    println!("{}", msg_type);
                }
            }
        }
        height -= 1;
        count -= 1;
    }
    Ok(())
}
