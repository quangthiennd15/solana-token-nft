use anchor_lang::prelude::*;
use instructions::*;

pub mod instructions;
pub mod constants;
pub mod errors;
pub mod states;
pub mod utils;

declare_id!("DdW7bY8dPn1WUgajNFcLgxCGJxKXLeGXKnKfo5v3vokB");

#[program]
pub mod soleague_marketplace {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, vault: Pubkey) -> Result<()> {
        spl_token::initialize(ctx, vault)
    }

    pub fn create_collection(ctx: Context<CreateCollection>, name: String, symbol: String, uri: String) -> Result<()> {
        spl_token::create_collection(ctx, name, symbol, uri)
    }

    pub fn mint_nft(ctx: Context<MintNft>, name: String, symbol: String, uri: String) -> Result<()> {
        spl_token::mint_nft(ctx, name, symbol, uri)
    }

    pub fn create_token(ctx: Context<CreateToken>, decimals: u8, name: String, symbol: String, uri: String) -> Result<()> {
        spl_token::create_token(ctx, decimals, name, symbol, uri)
    }

    pub fn mint_token(ctx: Context<MintToken>, amount: u64) -> Result<()> {
        spl_token::mint_token(ctx, amount)
    }

    pub fn burn_token(ctx: Context<BurnToken>, amount: u64) -> Result<()> {
        spl_token::burn_token(ctx, amount)
    }

    pub fn revoke_mint_authority(ctx: Context<RevokeMintAuthority>) -> Result<()> {
        spl_token::revoke_mint_authority(ctx)
    }

//     pub fn mint_nft_with_sig(ctx: Context<MintNft>, token_id: u64, total_price:u64, name: String, symbol: String, uri: String, eth_address: [u8; 20], sig: [u8; 64], recovery_id: u8) -> Result<()> {
//         spl_token::mint_nft_with_sig(ctx, token_id, total_price, name, symbol, uri, eth_address, sig, recovery_id)
//     }
//
//     pub fn verify_ed25519(ctx: Context<Verify>,  pubkey: Vec<u8>, msg: Vec<u8>, sig: Vec<u8>) -> Result<bool> {
//         signature::verify_ed25519(ctx, pubkey, msg, sig)
//     }

//     pub fn buy_node(ctx: Context<BuyNode>, request_id: u64, total_price: u64) -> Result<()> {
//          spl_token::buy_node(ctx,  request_id, total_price)
//      }

}

