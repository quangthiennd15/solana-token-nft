use anchor_lang::prelude::*;
use solana_program::instruction::Instruction;
use solana_program::sysvar::instructions::{ID as IX_ID, load_instruction_at_checked};

use crate::utils;

pub fn verify_ed25519(ctx: Context<Verify>,  pubkey: Vec<u8>, msg: Vec<u8>, sig: Vec<u8>) -> Result<bool> {
        // Get what should be the Ed25519Program instruction
        let ix: Instruction = load_instruction_at_checked(0, &ctx.accounts.ix_sysvar)?;

        // Check that ix is what we expect to have been sent
         let is_verified = utils::verify_ed25519_ix(&ix, &pubkey, &msg, &sig)?;

        // Do other stuff

        Ok(is_verified)
}

pub fn verify_secp(ctx: Context<Verify>, eth_address: [u8; 20], msg: Vec<u8>, sig: [u8; 64], recovery_id: u8) -> Result<()> {
        // Get what should be the Secp256k1Program instruction
        let ix: Instruction = load_instruction_at_checked(0, &ctx.accounts.ix_sysvar)?;

        // Check that ix is what we expect to have been sent
        utils::verify_secp256k1_ix(&ix, &eth_address, &msg, &sig, recovery_id)?;

        // Do other stuff

        Ok(())
}

#[derive(Accounts)]
pub struct Verify<'info> {
    pub sender: Signer<'info>,
    /// CHECK: ix_sysvar
    #[account(address = IX_ID)]
    pub ix_sysvar: AccountInfo<'info>,
}
