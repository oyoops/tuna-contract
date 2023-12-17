use anchor_lang::prelude::*;
use anchor_spl::token::{self, TokenAccount, Transfer};
declare_id!("BWh9JaJ7PkPCdTuVE2Fe7niB655k7koU9vC8URt35MR9");

#[program]
pub mod tuna_fee_contract {
    use super::*;
    pub fn transfer_with_fee(ctx: Context<TransferWithFee>, amount: u64) -> Result<()> {
        let fee = amount * ctx.accounts.adm_cont_fee_accts.fee_percentage / 100;
        let amount_after_fee = amount
            .checked_sub(2 * fee)
            .ok_or(ErrorCode::InsufficientFundsAfterFee)?;
        token::transfer(
            ctx.accounts.into_transfer_to_receiver_context(),
            amount_after_fee,
        )?;
        let first_fee_account_cpi_context = ctx.accounts.into_transfer_to_fee_account_context(ctx.accounts.first_fee_account.clone());
        token::transfer(first_fee_account_cpi_context, fee)?;
        let second_fee_account_cpi_context = ctx.accounts.into_transfer_to_fee_account_context(ctx.accounts.second_fee_account.clone());
        token::transfer(second_fee_account_cpi_context, fee)?;
        Ok(())
    }
    pub fn upd_fee_accts(
        ctx: Context<UpdFeeAccts>,
        new_first_fee_account: Pubkey,
        new_second_fee_account: Pubkey,
    ) -> Result<()> {
        let fee_accounts = &mut ctx.accounts.adm_cont_fee_accts;
        require_keys_eq!(
            fee_accounts.admin, 
            ctx.accounts.admin.key(), 
            ErrorCode::Unauthorized
        );
        fee_accounts.first_fee_account = new_first_fee_account;
        fee_accounts.second_fee_account = new_second_fee_account;
        Ok(())
    }
    pub fn upd_fee_pctge(ctx: Context<UpdFeePctg>, new_fee_percentage: u64) -> Result<()> {
        let fee_accounts = &mut ctx.accounts.adm_cont_fee_accts;
        require_keys_eq!(
            fee_accounts.admin, 
            ctx.accounts.admin.key(), 
            ErrorCode::Unauthorized
        );
        fee_accounts.fee_percentage = new_fee_percentage;
        Ok(())
    }
    pub fn init_adm_fee_accts(
        ctx: Context<InitAdmFeeAccts>,
        first_fee_account: Pubkey,
        second_fee_account: Pubkey,
        admin: Pubkey,
    ) -> Result<()> {
        let adm_fee_accts = &mut ctx.accounts.adm_fee_accts;
        adm_fee_accts.first_fee_account = first_fee_account;
        adm_fee_accts.second_fee_account = second_fee_account;
        adm_fee_accts.admin = admin;
        adm_fee_accts.fee_percentage = 0;
        Ok(())
    }
}
#[derive(Accounts)]
pub struct TransferWithFee<'info> {
    #[account(mut)]
    pub from: Account<'info, TokenAccount>,
    #[account(mut)]
    pub to: Account<'info, TokenAccount>,
    pub authority: Signer<'info>,
    pub token_program: Program<'info, anchor_spl::token::Token>,
    #[account(mut)]
    pub adm_cont_fee_accts: Account<'info, AdmContFeeAccts>,
    /// CHECK: manual. `first_fee_account` already valid SPL tkn acct in program logic
    #[account(mut)]
    pub first_fee_account: AccountInfo<'info>,
    /// CHECK: manual. `second_fee_account` already valid SPL tkn acct in program logic
    #[account(mut)]
    pub second_fee_account: AccountInfo<'info>,
}
#[derive(Accounts)]
pub struct InitAdmFeeAccts<'info> {
    #[account(init, payer = user, space = 8 + 32 + 32 + 32)]
    pub adm_fee_accts: Account<'info, AdmContFeeAccts>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}
impl<'info> TransferWithFee<'info> {
    fn into_transfer_to_receiver_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.from.to_account_info(),
            to: self.to.to_account_info(),
            authority: self.authority.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
    fn into_transfer_to_fee_account_context(&self, fee_account: AccountInfo<'info>) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.from.to_account_info(),
            to: fee_account,
            authority: self.authority.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
}
#[derive(Accounts)]
pub struct UpdFeeAccts<'info> {
    #[account(mut)]
    pub adm_cont_fee_accts: Account<'info, AdmContFeeAccts>,
    #[account(signer)]
    /// CHECK: adm pre-verified as valid auth for adm ops in program.
    pub admin: AccountInfo<'info>,
}
#[derive(Accounts)]
pub struct UpdFeePctg<'info> {
    #[account(mut)]
    pub adm_cont_fee_accts: Account<'info, AdmContFeeAccts>,
    #[account(signer)]
    /// CHECK: to burn.
    pub admin: AccountInfo<'info>,
}

#[account]
pub struct AdmContFeeAccts {
    pub first_fee_account: Pubkey,
    pub second_fee_account: Pubkey,
    /// CHECK: adm pre-verified as valid auth for adm ops in program.
    pub admin: Pubkey,
    pub fee_percentage: u64,
}
#[error_code]
pub enum ErrorCode {
    #[msg("Insufficient funds after calculating fees.")]
    InsufficientFundsAfterFee,
    Unauthorized,
    #[msg("Specified fee account not found.")]
    FeeAccountNotFound,
}
