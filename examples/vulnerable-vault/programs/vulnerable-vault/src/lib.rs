// ⚠️  DELIBERATELY VULNERABLE — FOR AUDIT DEMOS ONLY. DO NOT DEPLOY. ⚠️
//
// This is a teaching artifact for the solana-auditor skill. It contains
// intentional, clearly-marked vulnerabilities so you can demo the audit
// workflow end-to-end and confirm the skill finds them. The answer key is in
// examples/README.md. Every `VULN-N` tag maps to a pattern in
// skill/vulnerability-patterns.md.

use anchor_lang::prelude::*;

declare_id!("Vu1nVau1t11111111111111111111111111111111");

#[program]
pub mod vulnerable_vault {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        vault.authority = ctx.accounts.authority.key();
        vault.balance = 0;
        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        // VULN-9: unchecked arithmetic — `+` can overflow (no checked_add).
        vault.balance = vault.balance + amount;
        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        // VULN-10: integer division truncates the fee in the user's favor.
        // amount=99, fee_bps=100 -> fee = 0 (user pays no fee).
        let fee = amount * 100 / 10_000;
        let net = amount - fee; // VULN-9 again: can underflow if fee > amount paths change

        // VULN-1: `authority` is an UncheckedAccount with no is_signer check —
        // anyone can withdraw as if they were the vault authority.
        require_keys_eq!(
            vault.authority,
            ctx.accounts.authority.key(),
            VaultError::Unauthorized
        );
        vault.balance = vault.balance - net;
        Ok(())
    }

    pub fn set_authority(ctx: Context<SetAuthority>, new_authority: Pubkey) -> Result<()> {
        // VULN-21: privileged config change with no timelock and (see accounts)
        // no signer enforcement on the current authority.
        ctx.accounts.vault.authority = new_authority;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = authority, space = 8 + 32 + 8)]
    pub vault: Account<'info, Vault>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub vault: Account<'info, Vault>,
    pub depositor: Signer<'info>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub vault: Account<'info, Vault>,
    // VULN-1: should be `Signer<'info>`. As UncheckedAccount, no signature is
    // required, so the require_keys_eq above only checks the *pubkey matches*,
    // not that the real authority *signed*. An attacker passes the authority's
    // public key (which is public) without its private key.
    /// CHECK: deliberately unchecked for the demo
    pub authority: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct SetAuthority<'info> {
    // VULN-1: again no signer on the authority that may change control.
    #[account(mut)]
    pub vault: Account<'info, Vault>,
    /// CHECK: deliberately unchecked for the demo
    pub authority: UncheckedAccount<'info>,
}

#[account]
pub struct Vault {
    pub authority: Pubkey,
    pub balance: u64,
}

#[error_code]
pub enum VaultError {
    #[msg("Unauthorized")]
    Unauthorized,
}
