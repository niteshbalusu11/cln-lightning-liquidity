use std::str::FromStr;

use anyhow::bail;
use cln_rpc::primitives::PublicKey;
use rand::Rng;

#[derive(Debug)]
pub struct Uri {
    pub pubkey: PublicKey,
    pub host: String,
    pub port: u16,
}

pub fn decode_uri(s: &str) -> anyhow::Result<Uri> {
    let parts: Vec<&str> = s.split('@').collect();
    if parts.len() != 2 {
        bail!("Invalid URI format");
    }

    // Remove extra quotation marks and whitespace
    let pubkey_str = parts[0].trim_matches(|c: char| c == '\"' || c.is_whitespace());

    let pubkey = PublicKey::from_str(pubkey_str)?;

    let host_port: Vec<&str> = parts[1].split(':').collect();
    if host_port.len() != 2 {
        bail!("Invalid host:port format");
    }

    let host = host_port[0].to_string();
    let port: u16 = host_port[1].parse()?;

    Ok(Uri { pubkey, host, port })
}

pub fn make_id() -> String {
    let mut rng = rand::thread_rng();
    let bytes: [u8; 32] = rng.gen();

    let hex_string = hex::encode(bytes);

    hex_string
}
