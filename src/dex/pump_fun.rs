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
        signature::{Keypair, Signature},
        signer::Signer,
        system_program,
    },
    spl_associated_token_account::{get_associated_token_address, instruction::create_associated_token_account},
    spl_token::state::Account as TokenAccount,
    std::{str::FromStr, sync::Arc},
};

pub const PUMP_PROGRAM: &str = "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P";
pub const PUMP_GLOBAL: &str = "4wTV1YmiEkRvAtNtsSGPtUrqRYQMe5SKy2uB4Jjaxnjf";
pub const PUMP_FEE_RECIPIENT: &str = "CebN5WGQ4jvEPvsVU4EoHEpgzq1VV7AbicfhtW4xC9iM";
pub const PUMP_ACCOUNT: &str = "Ce6TQqeHC9p8KetsN6JsjHK7UTZk7nasjjnr7XxXp9F1";
pub const PUMP_BUY_METHOD: u64 = 16927863322537952870;
pub const PUMP_SELL_METHOD: u64 = 12502976635542562355;
pub const TOKEN_PROGRAM: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";

pub struct Pump {
    pub client: Arc<RpcClient>,
    pub keypair: Arc<Keypair>,
}

#[derive(BorshSerialize, BorshDeserialize)]
struct SwapInstruction {
    method: u8,
    amount: u64,
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
        if self.client.get_account(&token_account).await.is_err() {
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
        }

        Ok(token_account)
    }

    pub async fn swap(&self, mint: &str, config: SwapConfig) -> Result<Vec<String>> {
        let logger = Logger::new("[SWAP IN PUMP.FUN] => ".to_string());
        logger.log(format!("Swapping token: {}", mint));
        
        let mint_pubkey = Pubkey::from_str(mint)?;
        let wallet_pubkey = self.keypair.pubkey();
        let token_account = self.ensure_token_account(&mint_pubkey).await?;

        // Get bonding curve PDA
        let seeds = &[b"bonding_curve", mint_pubkey.as_ref()];
        let (bonding_curve, _) = Pubkey::find_program_address(
            seeds,
            &Pubkey::from_str(PUMP_PROGRAM)?
        );

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

pub async fn get_pump_info(
    rpc_client: Arc<solana_client::rpc_client::RpcClient>,
    mint: &str,
) -> Result<PumpInfo> {
    let mint_pubkey = Pubkey::from_str(mint)?;
    let program_id = Pubkey::from_str(PUMP_PROGRAM)?;
    
    // Get bonding curve PDA
    let seeds = &[b"bonding-curve".as_ref(), mint_pubkey.as_ref()];
    let (bonding_curve, _) = Pubkey::find_program_address(seeds, &program_id);

    // Get bonding curve account data
    let account = rpc_client.get_account(&bonding_curve)
        .map_err(|_| anyhow!("Bonding curve account not found"))?;

    // Deserialize account data
    let bonding_curve_data = BondingCurveAccount::try_from_slice(&account.data)
        .map_err(|_| anyhow!("Failed to deserialize bonding curve data"))?;

    // Get associated bonding curve
    let associated_seeds = &[b"associated-bonding-curve".as_ref(), mint_pubkey.as_ref()];
    let (associated_bonding_curve, _) = Pubkey::find_program_address(associated_seeds, &program_id);

    Ok(PumpInfo {
        mint: mint_pubkey.to_string(),
        bonding_curve: bonding_curve.to_string(),
        associated_bonding_curve: associated_bonding_curve.to_string(),
        raydium_pool: None, // TODO: Implement if needed
        raydium_info: None, // TODO: Implement if needed
        complete: bonding_curve_data.complete,
        virtual_sol_reserves: bonding_curve_data.virtual_sol_reserves,
        virtual_token_reserves: bonding_curve_data.virtual_token_reserves,
        total_supply: bonding_curve_data.token_total_supply,
    })
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

#[derive(Debug, BorshDeserialize)]
pub struct BondingCurveAccount {
    pub is_initialized: bool,
    pub bump: u8,
    pub mint: Pubkey,
    pub authority: Pubkey,
    pub complete: bool,
    pub virtual_sol_reserves: u64,
    pub virtual_token_reserves: u64,
    pub token_total_supply: u64,
}
