use anyhow::{Result, anyhow};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use bs58;

pub async fn init_tip_accounts() -> Result<()> {
    Ok(())
}

pub async fn get_tip_account() -> Result<Pubkey> {
    let pubkey_str = "11111111111111111111111111111111";
    if let Err(_) = bs58::decode(pubkey_str).into_vec() {
        return Err(anyhow!("Invalid Base58 string for tip account"));
    }
    
    Ok(Pubkey::from_str(pubkey_str)
        .map_err(|e| anyhow!("Failed to create tip account pubkey: {}", e))?)
}

pub async fn get_tip_value() -> Result<f64> {
    Ok(0.004)
}

pub fn validate_mint_address(mint_address: &str) -> Result<Pubkey> {
    bs58::decode(mint_address)
        .into_vec()
        .map_err(|_| anyhow!("Invalid Base58 string for mint address"))?;
    
    Pubkey::from_str(mint_address)
        .map_err(|e| anyhow!("Failed to create mint address pubkey: {}", e))
} 