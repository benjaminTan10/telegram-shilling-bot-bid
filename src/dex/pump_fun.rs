use {
    crate::common::{
        logger::Logger,
        utils::{SwapConfig, SwapDirection},
    },
    anyhow::{anyhow, Result},
    borsh::{BorshDeserialize, BorshSerialize},
    solana_client::nonblocking::rpc_client::RpcClient,
    solana_sdk::{
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
        signature::Keypair,
        signer::Signer,
        system_program,
    },
    spl_associated_token_account::{get_associated_token_address, instruction::create_associated_token_account},
    std::{str::FromStr, sync::Arc},
};

pub const PUMP_PROGRAM: &str = "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P";
pub const PUMP_GLOBAL: &str = "4wTV1YmiEkRvAtNtsSGPtUrqRYQMe5SKy2uB4Jjaxnjf";
pub const PUMP_FEE_RECIPIENT: &str = "CebN5WGQ4jvEPvsVU4EoHEpgzq1VV7AbicfhtW4xC9iM";
pub const PUMP_ACCOUNT: &str = "Ce6TQqeHC9p8KetsN6JsjHK7UTZk7nasjjnr7XxXp9F1";
pub const PUMP_BUY_METHOD: u64 = 16927863322537952870;
pub const PUMP_SELL_METHOD: u64 = 12502976635542562355;
pub const TOKEN_PROGRAM: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
pub const RENT_PROGRAM: &str = "SysvarRent111111111111111111111111111111111";
pub const ASSOCIATED_TOKEN_PROGRAM: &str = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";

pub struct Pump {
    pub client: Arc<RpcClient>,
    pub keypair: Arc<Keypair>,
}

#[derive(BorshSerialize, BorshDeserialize)]
struct SwapInstruction {
    method: u8,
    amount: u64,
}

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct BondingCurveAccount {
    pub is_initialized: bool,
    pub bump: u8,
    pub mint: Pubkey,
    pub authority: Pubkey,
    pub complete: bool,
    pub virtual_sol_reserves: u64,
    pub virtual_token_reserves: u64,
    pub real_token_reserves: u64,
    pub real_sol_reserves: u64,
    pub token_total_supply: u64,
}

impl Pump {
    pub fn new(client: Arc<RpcClient>, keypair: Arc<Keypair>) -> Self {
        Self { client, keypair }
    }

    pub async fn get_token_balance(&self, mint: &str) -> Result<u64> {
        let mint_pubkey = Pubkey::from_str(mint)?;
        let wallet_pubkey = self.keypair.pubkey();
        let token_account = get_associated_token_address(&wallet_pubkey, &mint_pubkey);
        
        let balance = self.client.get_token_account_balance(&token_account).await?;
        Ok(balance.amount.parse()?)
    }

    async fn ensure_token_account(&self, mint: &Pubkey) -> Result<Pubkey> {
        let wallet_pubkey = self.keypair.pubkey();
        let token_account = get_associated_token_address(&wallet_pubkey, mint);

        // Check if account exists
        match self.client.get_account(&token_account).await {
            Ok(_) => Ok(token_account),
            Err(_) => {
                // Create ATA if it doesn't exist
                let create_ata_ix = create_associated_token_account(
                    &wallet_pubkey,
                    &wallet_pubkey,
                    mint,
                    &Pubkey::from_str(TOKEN_PROGRAM)?,
                );

                let recent_blockhash = self.client.get_latest_blockhash().await?;
                let transaction = solana_sdk::transaction::Transaction::new_signed_with_payer(
                    &[create_ata_ix],
                    Some(&wallet_pubkey),
                    &[&*self.keypair],
                    recent_blockhash,
                );

                self.client.send_and_confirm_transaction(&transaction).await?;
                Ok(token_account)
            }
        }
    }

    pub async fn buy(&self, mint: &str, amount: u64) -> Result<String> {
        let config = SwapConfig {
            amount,
            swap_direction: SwapDirection::Buy,
            slippage: 100, // 1% slippage
            use_jito: false,
        };

        let signatures = self.swap(mint, config).await?;
        Ok(signatures[0].clone())
    }

    pub async fn sell(&self, mint: &str, amount: u64) -> Result<String> {
        let config = SwapConfig {
            amount,
            swap_direction: SwapDirection::Sell,
            slippage: 100, // 1% slippage
            use_jito: false,
        };

        let signatures = self.swap(mint, config).await?;
        Ok(signatures[0].clone())
    }

    pub async fn swap(&self, mint: &str, config: SwapConfig) -> Result<Vec<String>> {
        let logger = Logger::new("[SWAP IN PUMP.FUN] => ".to_string());
        
        // Validate minimum amount
        if config.amount == 0 {
            return Err(anyhow!("Amount cannot be zero"));
        }

        // For buys, check if we have enough SOL
        if matches!(config.swap_direction, SwapDirection::Buy) {
            let wallet_balance = self.client.get_balance(&self.keypair.pubkey()).await?;
            if wallet_balance <= config.amount {
                return Err(anyhow!("Insufficient SOL balance for buy"));
            }
        }

        // For sells, check if we have enough tokens
        if matches!(config.swap_direction, SwapDirection::Sell) {
            let token_balance = self.get_token_balance(mint).await?;
            if token_balance < config.amount {
                return Err(anyhow!("Insufficient token balance for sell"));
            }
        }

        logger.log(format!(
            "Swapping token: {} - Amount: {} - Direction: {:?}",
            mint, config.amount, config.swap_direction
        ));
        
        let mint_pubkey = Pubkey::from_str(mint)?;
        let wallet_pubkey = self.keypair.pubkey();
        let token_account = self.ensure_token_account(&mint_pubkey).await?;

        // Get bonding curve PDA and account data
        let program_id = Pubkey::from_str(PUMP_PROGRAM)?;
        let (bonding_curve, _) = Pubkey::find_program_address(
            &[b"bonding_curve", mint_pubkey.as_ref()],
            &program_id
        );

        // Get bonding curve account data
        let account = self.client.get_account(&bonding_curve)
            .await
            .map_err(|_| anyhow!("Failed to get bonding curve account"))?;

        let bonding_curve_data = BondingCurveAccount::try_from_slice(&account.data)
            .map_err(|_| anyhow!("Failed to deserialize bonding curve data"))?;

        // Build instruction data
        let method = match config.swap_direction {
            SwapDirection::Buy => 0u8,
            SwapDirection::Sell => 1u8,
        };

        let instruction_data = SwapInstruction {
            method,
            amount: config.amount,
        };

        let data = borsh::to_vec(&instruction_data)?;

        // Build instruction
        let instruction = Instruction {
            program_id: Pubkey::from_str(PUMP_PROGRAM)?,
            accounts: vec![
                AccountMeta::new(wallet_pubkey, true),
                AccountMeta::new(token_account, false),
                AccountMeta::new(bonding_curve, false),
                AccountMeta::new_readonly(mint_pubkey, false),
                AccountMeta::new_readonly(Pubkey::from_str(PUMP_GLOBAL)?, false),
                AccountMeta::new(Pubkey::from_str(PUMP_FEE_RECIPIENT)?, false),
                AccountMeta::new_readonly(system_program::ID, false),
                AccountMeta::new_readonly(Pubkey::from_str(TOKEN_PROGRAM)?, false),
            ],
            data,
        };

        // Send transaction
        let recent_blockhash = self.client.get_latest_blockhash().await?;
        let transaction = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &[instruction],
            Some(&wallet_pubkey),
            &[&*self.keypair],
            recent_blockhash,
        );

        let signature = self.client.send_and_confirm_transaction(&transaction).await?;
        Ok(vec![signature.to_string()])
    }
}

pub async fn get_bonding_curve_account(
    rpc_client: Arc<solana_client::rpc_client::RpcClient>,
    mint: &Pubkey,
    program_id: &Pubkey,
) -> Result<(Pubkey, Pubkey, BondingCurveAccount)> {
    tracing::info!("get_bonding_curve_account called with mint: {} and program_id: {}", mint, program_id);
    // Get bonding curve PDA with correct seeds
    let seeds = &[b"bonding-curve", mint.as_ref()];
    let (bonding_curve, _bump) = Pubkey::find_program_address(seeds, program_id);

    tracing::info!(
        "Looking up bonding curve: {} for mint: {}",
        bonding_curve.to_string(),
        mint.to_string()
    );

    // Get associated token account
    let associated_bonding_curve = get_associated_token_address(&bonding_curve, mint);

    // Get account data with retries and proper error handling
    let mut retries = 3;
    while retries > 0 {
        match rpc_client.get_account(&bonding_curve) {
            Ok(account) => {
                tracing::info!("Found bonding curve account with {} bytes", account.data.len());
                
                // Skip 8 bytes of discriminator
                let data = if account.data.len() > 8 {
                    &account.data[8..]
                } else {
                    return Err(anyhow!("Account data too short"));
                };

                match BondingCurveAccount::try_from_slice(data) {
                    Ok(bonding_curve_data) => {
                        // Validate the account data
                        if bonding_curve_data.mint != *mint {
                            return Err(anyhow!("Bonding curve mint mismatch"));
                        }

                        return Ok((bonding_curve, associated_bonding_curve, bonding_curve_data));
                    }
                    Err(e) => {
                        tracing::warn!("Failed to deserialize account data: {}", e);
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Failed to get account: {}", e);
            }
        }
        retries -= 1;
        if retries > 0 {
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    }

    Err(anyhow!("Failed to get valid bonding curve account after retries"))
}

pub async fn get_pump_info(
    rpc_client: Arc<solana_client::rpc_client::RpcClient>,
    mint: &str,
) -> Result<PumpInfo> {
    let mint_pubkey = Pubkey::from_str(mint)?;
    let program_id = Pubkey::from_str(PUMP_PROGRAM)?;
    tracing::info!("get_pump_info  function called.");
    let logger = Logger::new("[get_pump_info TX]".to_string());
    logger.info(format!(
        "\n  get_pump_info  function called. ---"
    ));

    match get_bonding_curve_account(rpc_client.clone(), &mint_pubkey, &program_id).await {
        Ok((bonding_curve, associated_bonding_curve, account)) => {
            Ok(PumpInfo {
                mint: mint.to_string(),
                bonding_curve: bonding_curve.to_string(),
                associated_bonding_curve: associated_bonding_curve.to_string(),
                raydium_pool: None,
                raydium_info: None,
                complete: account.complete,
                virtual_sol_reserves: account.virtual_sol_reserves,
                virtual_token_reserves: account.virtual_token_reserves,
                total_supply: account.token_total_supply,
            })
        }
        Err(e) => {
            tracing::error!("Failed to get bonding curve account: {}", e);
            Err(anyhow!("Failed to get bonding curve account: {}", e))
        }
    }
}

pub struct PumpInfo {
    pub mint: String,
    pub bonding_curve: String,
    pub associated_bonding_curve: String,
    pub raydium_pool: Option<String>,
    pub raydium_info: Option<String>,
    pub complete: bool,
    pub virtual_sol_reserves: u64,
    pub virtual_token_reserves: u64,
    pub total_supply: u64,
}

pub const PUMP_PROGRAM_ID: &str = "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P";

#[derive(BorshSerialize, BorshDeserialize)]
struct BuyInstruction {
    amount: u64,
}

#[derive(BorshSerialize, BorshDeserialize)]
struct SellInstruction {
    amount: u64,
}

fn get_bonding_curve(mint: &Pubkey) -> Result<Pubkey> {
    let seeds = &[b"bonding-curve".as_ref(), mint.as_ref()];
    let (bonding_curve, _) = Pubkey::find_program_address(seeds, &Pubkey::from_str(PUMP_PROGRAM)?);
    Ok(bonding_curve)
}
