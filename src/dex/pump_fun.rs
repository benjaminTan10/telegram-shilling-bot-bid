use {
    crate::common::logger::Logger,
    anyhow::{anyhow, Result},
    anchor_client::{
        solana_sdk::{
            signature::Keypair,
            signer::Signer,
        },
        Cluster,
    },
    borsh::{BorshDeserialize, BorshSerialize},
    pumpfun::{
        accounts::BondingCurveAccount,
        PriorityFee,
        PumpFun as PumpFunClient,
        error::ClientError,
    },
    solana_client::nonblocking::rpc_client::RpcClient,
    solana_sdk::pubkey::Pubkey,
    std::{str::FromStr, sync::Arc},
    spl_associated_token_account::instruction::create_associated_token_account,
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
    pump_client: PumpFunClient,
}

impl Pump {
    pub fn new(client: Arc<RpcClient>, keypair: Arc<Keypair>) -> Self {
        let pump_client = PumpFunClient::new(
            Cluster::Mainnet,
            keypair.clone(),
            None,
            None,
        );

        Self { 
            client,
            keypair,
            pump_client,
        }
    }

    pub async fn ensure_token_account(&self, mint: &str) -> Result<()> {
        let mint_pubkey = Pubkey::from_str(mint)?;
        let wallet_pubkey = self.keypair.pubkey();
        let token_account = spl_associated_token_account::get_associated_token_address(
            &wallet_pubkey,
            &mint_pubkey
        );
        
        // Check if account exists
        match self.client.get_account(&token_account).await {
            Ok(_) => Ok(()),
            Err(_) => {
                // Create ATA if it doesn't exist
                let create_ata_ix = create_associated_token_account(
                    &self.keypair.pubkey(),
                    &self.keypair.pubkey(),
                    &mint_pubkey,
                    &spl_token::id(),
                );
                
                let recent_blockhash = self.client.get_latest_blockhash().await?;
                let transaction = solana_sdk::transaction::Transaction::new_signed_with_payer(
                    &[create_ata_ix],
                    Some(&self.keypair.pubkey()),
                    &[&*self.keypair],
                    recent_blockhash,
                );
                
                self.client.send_and_confirm_transaction(&transaction).await?;
                
                // Verify account was created
                self.client.get_account(&token_account).await
                    .map_err(|_| anyhow!("Failed to verify token account creation"))?;
                
                Ok(())
            }
        }
    }

    pub async fn get_token_balance(&self, mint: &str) -> Result<u64> {
        let mint_pubkey = Pubkey::from_str(mint)?;
        let wallet_pubkey = self.keypair.pubkey();
        let token_account = spl_associated_token_account::get_associated_token_address(
            &wallet_pubkey,
            &mint_pubkey
        );
        
        // Ensure token account exists
        self.ensure_token_account(mint).await?;
        
        // Get balance, return 0 if account not found
        match self.client.get_token_account_balance(&token_account).await {
            Ok(balance) => Ok(balance.amount.parse()?),
            Err(_) => Ok(0)
        }
    }

    pub async fn buy(&self, mint: &str, amount: u64) -> Result<String> {
        // Don't try to buy if amount is 0
        if amount == 0 {
            return Err(anyhow!("Cannot buy with 0 SOL"));
        }

        // Ensure we have enough SOL balance
        let wallet_balance = self.client.get_balance(&self.keypair.pubkey()).await?;
        if wallet_balance < amount {
            return Err(anyhow!("Insufficient SOL balance: have {}, need {}", wallet_balance, amount));
        }

        // Ensure token account exists before buying
        self.ensure_token_account(mint).await?;

        let mint_pubkey = Pubkey::from_str(mint)?;
        let fee = Some(PriorityFee {
            limit: Some(100_000),
            price: Some(100_000_000),
        });

        let signature = self.pump_client.buy(&mint_pubkey, amount, None, fee)
            .await
            .map_err(|e: ClientError| anyhow!("Buy failed: {}", e))?;
        Ok(signature.to_string())
    }

    pub async fn sell(&self, mint: &str, amount: u64) -> Result<String> {
        // Don't try to sell if amount is 0
        if amount == 0 {
            return Err(anyhow!("Cannot sell 0 tokens"));
        }

        // Ensure token account exists and we have enough balance
        let current_balance = self.get_token_balance(mint).await?;
        if current_balance < amount {
            return Err(anyhow!("Insufficient token balance: have {}, need {}", current_balance, amount));
        }

        let mint_pubkey = Pubkey::from_str(mint)?;
        let fee = Some(PriorityFee {
            limit: Some(100_000),
            price: Some(100_000_000),
        });

        let signature = self.pump_client.sell(&mint_pubkey, Some(amount), None, fee)
            .await
            .map_err(|e: ClientError| anyhow!("Sell failed: {}", e))?;
        Ok(signature.to_string())
    }
}

pub async fn get_bonding_curve_account(
    _rpc_client: Arc<solana_client::rpc_client::RpcClient>,
    mint: &Pubkey,
    _program_id: &Pubkey,
) -> Result<(Pubkey, Pubkey, BondingCurveAccount)> {
    let _logger = Logger::new("[get_bonding_curve_account TX]".to_string());
    
    // Create PumpFun client
    let payer = Arc::new(Keypair::new());
    let pump_client = PumpFunClient::new(Cluster::Mainnet, payer, None, None);
    
    // Get bonding curve PDA
    let bonding_curve = PumpFunClient::get_bonding_curve_pda(mint)
        .ok_or_else(|| anyhow!("Failed to derive bonding curve PDA"))?;

    let associated_bonding_curve = spl_associated_token_account::get_associated_token_address(
        &bonding_curve,
        mint
    );

    // Get account data
    let bonding_curve_account = pump_client.get_bonding_curve_account(mint)
        .await
        .map_err(|e: ClientError| anyhow!("Failed to get bonding curve account: {}", e))?;

    Ok((bonding_curve, associated_bonding_curve, bonding_curve_account))
}

pub async fn get_pump_info(
    rpc_client: Arc<solana_client::rpc_client::RpcClient>,
    mint: &str,
) -> Result<PumpInfo> {
    let mint_pubkey = Pubkey::from_str(mint)?;
    let program_id = Pubkey::from_str(PUMP_PROGRAM)?;

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
        Err(e) => Err(anyhow!("Failed to get bonding curve account: {}", e))
    }
}

#[derive(Debug)]
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