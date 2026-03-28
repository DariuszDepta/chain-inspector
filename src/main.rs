use base64::Engine;
use base64::engine::general_purpose;

const NEUTRON_RPC: &str = "https://rpc-lb.neutron.org:443";
const OSMOSIS_RPC: &str = "https://osmosis-rpc.publicnode.com:443";
const COSMOS_RPC: &str = "https://cosmos-rpc.publicnode.com:443";

/// Gets the height of the latest block.
///
/// Equivalent of:
///
/// ```shell
/// curl -s "https://rpc-lb.neutron.org:443/status" \
/// | jq -r '.result.sync_info.latest_block_height'
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
/// curl -s "https://rpc-lb.neutron.org:443/tx_search?query=\"tx.height=51923449\"" \
/// | jq '[.result.txs[].hash]'
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
async fn get_msg_types(url: &str, hash: &str) -> anyhow::Result<(Vec<String>, usize)> {
  let mut msg_types = vec![];
  let command = format!("{}/tx?hash=0x{}", url, hash);
  let response: serde_json::Value = reqwest::get(command).await?.json().await?;
  let Some(events) = &response["result"]["tx_result"]["events"].as_array() else {
    return Ok((msg_types, 0));
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
  if !msg_types.is_empty() {
    let tx = &response["result"]["tx"].as_str().unwrap().to_string();
    let bytes: Vec<u8> = general_purpose::STANDARD.decode(tx).expect("invalid base64");
    let length = bytes.len();
    Ok((msg_types, length))
  } else {
    Ok((msg_types, 0))
  }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let args = std::env::args().skip(1).collect::<Vec<String>>();
  if args.is_empty() {
    eprintln!("missing argument, expected: neutron, osmosis or cosmos");
    std::process::exit(1);
  }
  if args.len() > 1 {
    eprintln!("too many arguments, expected: neutron, osmosis or cosmos");
    std::process::exit(1);
  }
  let name = args[0].as_str();
  let url = match name {
    "neutron" => NEUTRON_RPC,
    "osmosis" => OSMOSIS_RPC,
    "cosmos" => COSMOS_RPC,
    _ => {
      eprintln!("invalid argument, expected: neutron, osmosis or cosmos");
      std::process::exit(1);
    }
  };
  let mut height = get_latest_block_height(url).await?;
  let mut max_size = 0;
  let mut count = 1;
  while height > 0 {
    let hashes = get_transaction_hashes(url, height).await?;
    if !hashes.is_empty() {
      for hash in &hashes {
        let (msg_types, size) = get_msg_types(url, hash).await?;
        if !msg_types.is_empty() {
          for msg_type in &msg_types {
            println!("  {}", msg_type);
          }
          if size > max_size {
            max_size = size;
          }
          println!("{:20} {:20} {:20} {:20} {}", height, max_size, size, count, hash);
        }
      }
    }
    height -= 1;
    count += 1;
  }
  Ok(())
}
