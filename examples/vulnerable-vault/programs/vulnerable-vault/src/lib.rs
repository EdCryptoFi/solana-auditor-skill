// ⚠️  DELIBERATELY VULNERABLE — FOR AUDIT DEMOS ONLY. DO NOT DEPLOY. ⚠️
//
// This is a teaching artifact for the solana-auditor skill. It contains
// intentional, clearly-marked vulnerabilities so you can demo the audit
// workflow end-to-end and confirm the skill finds them. The answer key is in
// examples/README.md. Every `VULN-N` tag maps to a pattern number in
// skill/vulnerability-patterns.md.

use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke;
use anchor_lang::solana_program::instruction::{AccountMeta, Instruction};

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
        // VULN-9: unchecked arithmetic — `+` can overflow (no checked_add),
        // and overflow-checks is unset in Cargo.toml.
        vault.balance = vault.balance + amount;
        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        // VULN-10: integer division truncates the fee in the user's favor.
        // amount=99, fee_bps=100 -> fee = 0 (user pays no fee).
        let fee = amount * 100 / 10_000;
        let net = amount - fee; // VULN-9: can underflow on other call paths.

        // VULN-1: `authority` is an UncheckedAccount with no is_signer check —
        // require_keys_eq only checks the pubkey matches (pubkeys are public),
        // not that the real authority *signed*. Anyone can drain the vault.
        require_keys_eq!(
            vault.authority,
            ctx.accounts.authority.key(),
            VaultError::Unauthorized
        );
        vault.balance = vault.balance - net;
        Ok(())
    }

    // Internal balance move between two sub-accounts of the same vault.
    pub fn transfer_internal(ctx: Context<TransferInternal>, amount: u64) -> Result<()> {
        // VULN-5: no constraint preventing `from` == `to`. If the caller passes
        // the same account for both, the credit can erase the debit (state
        // aliasing), or balances desync.
        let from = &mut ctx.accounts.from;
        let to = &mut ctx.accounts.to;
        from.balance = from.balance - amount;
        to.balance = to.balance + amount;
        Ok(())
    }

    // Reads a config account that was created by a *different* program.
    pub fn apply_config(ctx: Context<ApplyConfig>) -> Result<()> {
        // VULN-2: raw deserialization with NO owner check. An attacker passes an
        // account they own, laid out like Config, to inject arbitrary values.
        let data = ctx.accounts.config.try_borrow_data()?;
        let cfg = Config::try_from_slice(&data[8..])?;
        ctx.accounts.vault.fee_bps = cfg.fee_bps;
        Ok(())
    }

    // Performs a CPI to a "router" program supplied by the caller.
    pub fn route_withdraw(ctx: Context<RouteWithdraw>, amount: u64) -> Result<()> {
        // VULN-3: arbitrary CPI — the target program is taken from the caller
        // and never validated against an expected program ID. Attacker passes a
        // malicious program that signs/forwards however it likes.
        let ix = Instruction {
            program_id: ctx.accounts.router_program.key(),
            accounts: vec![AccountMeta::new(ctx.accounts.vault.key(), false)],
            data: amount.to_le_bytes().to_vec(),
        };
        invoke(
            &ix,
            &[
                ctx.accounts.vault.to_account_info(),
                ctx.accounts.router_program.to_account_info(),
            ],
        )?;
        Ok(())
    }

    pub fn close_position(ctx: Context<ClosePosition>) -> Result<()> {
        // VULN-8: insecure account closing — lamports are drained but the data
        // is NOT zeroed and the owner is not reassigned. Within the same
        // transaction the stale data is still readable / re-usable.
        let pos = ctx.accounts.position.to_account_info();
        let dest = ctx.accounts.receiver.to_account_info();
        **dest.try_borrow_mut_lamports()? += **pos.try_borrow_lamports()?;
        **pos.try_borrow_mut_lamports()? = 0;
        // (no data zeroing, no Anchor `close = receiver`)
        Ok(())
    }

    pub fn set_authority(ctx: Context<SetAuthority>, new_authority: Pubkey) -> Result<()> {
        // VULN-21: privileged config change with no timelock — and (see accounts)
        // no signer enforcement on the current authority. Instant takeover.
        ctx.accounts.vault.authority = new_authority;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = authority, space = 8 + 32 + 8 + 2)]
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
    // required, so the require_keys_eq in the handler is satisfiable by anyone.
    /// CHECK: deliberately unchecked for the demo
    pub authority: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct TransferInternal<'info> {
    #[account(mut)]
    pub from: Account<'info, Vault>,
    #[account(mut)]
    pub to: Account<'info, Vault>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct ApplyConfig<'info> {
    #[account(mut)]
    pub vault: Account<'info, Vault>,
    // VULN-2: AccountInfo with no owner constraint; data is trusted raw.
    /// CHECK: deliberately unchecked for the demo
    pub config: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct RouteWithdraw<'info> {
    #[account(mut)]
    pub vault: Account<'info, Vault>,
    // VULN-3: caller-supplied program, never validated.
    /// CHECK: deliberately unchecked for the demo
    pub router_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct ClosePosition<'info> {
    // VULN-8: manual close path instead of Anchor's `close = receiver`.
    #[account(mut)]
    pub position: Account<'info, Vault>,
    /// CHECK: lamport destination
    #[account(mut)]
    pub receiver: AccountInfo<'info>,
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
    pub fee_bps: u16,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Config {
    pub fee_bps: u16,
}

#[error_code]
pub enum VaultError {
    #[msg("Unauthorized")]
    Unauthorized,
}
