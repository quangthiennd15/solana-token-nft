use anchor_lang::prelude::*;

#[account]
pub struct Init{
    pub admin: Pubkey,
    pub vault: Pubkey,
}