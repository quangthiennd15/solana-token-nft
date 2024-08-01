use anchor_lang::prelude::*;
use anchor_lang::solana_program::system_instruction;
use anchor_spl::{token, metadata};
use anchor_spl::token::{Token, MintTo, TokenAccount, Mint, SetAuthority, spl_token::instruction::AuthorityType, Burn};
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::metadata::{CreateMasterEditionV3, CreateMetadataAccountsV3, Metadata, SetAndVerifySizedCollectionItem, SignMetadata};
use mpl_token_metadata::types::{CollectionDetails, Collection, Creator, DataV2};
use crate::states::{init};
use crate::utils;
use crate::constants::{ COLLECTION_SEED, EDITION_SEED, INIT_SEED, METADATA_SEED, COLLECTION_INFO, TOKEN_SEED};
use crate::errors::{MarketPlaceError, SigError};
use solana_program::instruction::Instruction;
use solana_program::sysvar::instructions::{ID as IX_ID, load_instruction_at_checked};
use solana_program::keccak;

fn find_metadata_account(mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            METADATA_SEED,
            mpl_token_metadata::ID.as_ref(),
            mint.as_ref(),
        ], &mpl_token_metadata::ID)
}

fn find_master_edition_account(mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            METADATA_SEED,
            mpl_token_metadata::ID.as_ref(),
            mint.as_ref(),
            EDITION_SEED
        ], &mpl_token_metadata::ID)
}

pub fn initialize(ctx: Context<Initialize>, vault: Pubkey) -> Result<()> {
    let init = &mut ctx.accounts.init;
    init.admin = ctx.accounts.admin.key();
    init.vault = vault;
    Ok(())
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init, //init
        payer = admin,
        space = 32 + 32 +32,
        seeds = [INIT_SEED],
        bump,
    )]
    pub init: Account<'info, init::Init>,
    pub system_program: Program<'info, System>
}

pub fn create_collection(ctx: Context<CreateCollection>, name: String, symbol: String, uri: String) -> Result<()> {
    let signer_seeds : &[&[&[u8]]] = &[&[COLLECTION_SEED, &[ctx.bumps.collection_mint]]];

    msg!("minting !");

    token::mint_to(CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        MintTo{
            mint: ctx.accounts.collection_mint.to_account_info(),
            to: ctx.accounts.token_account.to_account_info(),
            authority: ctx.accounts.collection_mint.to_account_info(),
        },
        signer_seeds), 1)?;

    msg!("creating metadata account");

    let data_v2 = DataV2{
        name: name,
        symbol: symbol,
        uri: uri,
        seller_fee_basis_points: 0,
        creators: Some(vec![Creator{
            address: ctx.accounts.admin.key(),
            verified: false,
            share: 100
        }]),
        collection: None,
        uses: None,
    };

    metadata::create_metadata_accounts_v3(
        CpiContext::new_with_signer(
            ctx.accounts.metadata_program.to_account_info(),
            CreateMetadataAccountsV3 {
                metadata: ctx.accounts.metadata_account.to_account_info(),
                mint: ctx.accounts.collection_mint.to_account_info(),
                mint_authority: ctx.accounts.collection_mint.to_account_info(),
                payer: ctx.accounts.admin.to_account_info(),
                update_authority: ctx.accounts.collection_mint.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info()
            },
            signer_seeds),
        data_v2,
        true,
        true,
        Some(CollectionDetails::V1 { size: 0 }),
    )?;

    // create master edition account for collection nft
    metadata::create_master_edition_v3(
        CpiContext::new_with_signer(
            ctx.accounts.metadata_program.to_account_info(),
                CreateMasterEditionV3 {
                    payer: ctx.accounts.admin.to_account_info(),
                    mint: ctx.accounts.collection_mint.to_account_info(),
                    edition: ctx.accounts.master_edition.to_account_info(),
                    mint_authority: ctx.accounts.collection_mint.to_account_info(),
                    update_authority: ctx.accounts.collection_mint.to_account_info(),
                    metadata: ctx.accounts.metadata_account.to_account_info(),
                    token_program: ctx.accounts.token_program.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
                &signer_seeds,
            ),
               Some(0),
    )?;

    msg!("attach metadata successfully ! and creating sign metadata account");

    metadata::sign_metadata(CpiContext::new(
        ctx.accounts.metadata_program.to_account_info(),
        SignMetadata {
            creator: ctx.accounts.admin.to_account_info(),
            metadata: ctx.accounts.metadata_account.to_account_info()
        }
    ))?;
    Ok(())
}

#[derive(Accounts)]
pub struct CreateCollection<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
            init_if_needed,
            seeds = [COLLECTION_SEED],
            bump,
            payer = admin,
            mint::decimals = 0,
            mint::authority = collection_mint,
            mint::freeze_authority = collection_mint
    )]
    pub collection_mint: Account<'info, Mint>,

    #[account(
        init_if_needed,
        payer = admin,
        associated_token::mint = collection_mint,
        associated_token::authority = admin,
    )]
    pub token_account: Box<Account<'info, TokenAccount>>,

     /// CHECK: address
     #[account(
        mut,
        address= find_metadata_account(&collection_mint.key()).0
    )]
    pub metadata_account: UncheckedAccount<'info>,

     /// CHECK: address
     #[account(
        mut,
        address= find_master_edition_account(&collection_mint.key()).0
    )]
    pub master_edition: UncheckedAccount<'info>,

    pub rent: Sysvar<'info, Rent>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub metadata_program: Program<'info, Metadata>,
    pub system_program: Program<'info, System>
}

pub fn mint_nft(ctx: Context<MintNft>, name: String, symbol: String, uri: String) -> Result<()> {
    msg!("minting !");
    let signer_seeds: &[&[&[u8]]] = &[&[COLLECTION_SEED, &[ctx.bumps.collection_mint]]];

    token::mint_to(CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        MintTo{
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.nft_account_to.to_account_info(),
            authority: ctx.accounts.collection_mint.to_account_info()
        }, signer_seeds), 1)?;

    msg!("minted nft successfully and creating metadata account v3");

    metadata::create_metadata_accounts_v3(CpiContext::new_with_signer(
        ctx.accounts.metadata_program.to_account_info(),
        metadata::CreateMetadataAccountsV3{
            metadata: ctx.accounts.metadata_account.to_account_info(),
            mint: ctx.accounts.mint.to_account_info(),
            mint_authority: ctx.accounts.collection_mint.to_account_info(),
            update_authority: ctx.accounts.collection_mint.to_account_info(),
            payer: ctx.accounts.user.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            rent: ctx.accounts.rent.to_account_info(),
        }, signer_seeds
    ), DataV2{
        name: name,
        symbol: symbol,
        uri: uri,
        seller_fee_basis_points: 0,
        creators: None,
        collection: None,
        uses: None,
    }, true, true, None)?;

    msg!("minted nft created metadata successfully and creating master edtion account");

    // create master edition account for nft in collection
    metadata::create_master_edition_v3(
        CpiContext::new_with_signer(
            ctx.accounts.metadata_program.to_account_info(),
            CreateMasterEditionV3 {
                payer: ctx.accounts.user.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                edition: ctx.accounts.master_edition.to_account_info(),
                mint_authority: ctx.accounts.collection_mint.to_account_info(),
                update_authority: ctx.accounts.collection_mint.to_account_info(),
                metadata: ctx.accounts.metadata_account.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
            &signer_seeds,
        ),
        Some(0),
    )?;

    metadata::set_and_verify_sized_collection_item(CpiContext::new_with_signer(
        ctx.accounts.metadata_program.to_account_info(),
        SetAndVerifySizedCollectionItem  {
            metadata: ctx.accounts.metadata_account.to_account_info(),
            collection_authority: ctx.accounts.collection_mint.to_account_info(),
            payer: ctx.accounts.user.to_account_info(),
            update_authority: ctx.accounts.collection_mint.to_account_info(),
            collection_mint: ctx.accounts.collection_mint.to_account_info(),
            collection_metadata: ctx.accounts.collection_metadata_account.to_account_info(),
            collection_master_edition: ctx.accounts.collection_master_edition_account.to_account_info(),
        }, signer_seeds
    ), None)?;

    Ok(())
}

#[derive(Accounts)]
pub struct MintNft<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    /// CHECK: address
    #[account(
        mut,
        seeds = [COLLECTION_SEED],
        bump,
    )]
    pub collection_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = user,
        mint::decimals = 0,
        mint::authority = collection_mint,
        mint::freeze_authority = collection_mint,
    )]
    pub mint: Box<Account<'info, Mint>>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = mint,
        associated_token::authority = user,
    )]
    pub nft_account_to: Account<'info, TokenAccount>,

     /// CHECK: address
     #[account(
        mut,
        address= find_metadata_account(&collection_mint.key()).0
    )]
    pub collection_metadata_account: UncheckedAccount<'info>,

     /// CHECK: address
     #[account(
        mut,
        address= find_master_edition_account(&collection_mint.key()).0
    )]
    pub collection_master_edition_account: UncheckedAccount<'info>,

     /// CHECK:
     #[account(
         mut,
         address = find_master_edition_account(&mint.key()).0
     )]
     pub master_edition: UncheckedAccount<'info>,

     /// CHECK: address
     #[account(
        mut,
        address = find_metadata_account(&mint.key()).0
    )]
    pub metadata_account: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub metadata_program: Program<'info, Metadata>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn create_token(ctx: Context<CreateToken>, decimals: u8, name: String, symbol: String, uri: String) -> Result<()>{
    let signer_seeds: &[&[&[u8]]] = &[&[TOKEN_SEED, &[ctx.bumps.mint]]];

    msg!("creating metadata account");
    let data_v2 = DataV2{
        name: name,
        symbol: symbol,
        uri: uri,
        seller_fee_basis_points: 0,
        creators: Some(vec![Creator{
            address: ctx.accounts.admin.key(),
            verified: false,
            share: 100
        }]),
        collection: None,
        uses: None,
    };

    metadata::create_metadata_accounts_v3(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            metadata::CreateMetadataAccountsV3{
                metadata: ctx.accounts.metadata_account.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                mint_authority: ctx.accounts.mint.to_account_info(), //mint authority of mint account
                payer: ctx.accounts.admin.to_account_info(),
                update_authority: ctx.accounts.mint.to_account_info(), // update authority of metadata account
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
            signer_seeds
        ),
        data_v2,
        true,
        true,
        None
    )?;

    msg!("attach metadata successfully and creating sign metadata account");

    metadata::sign_metadata(CpiContext::new(
        ctx.accounts.metadata_program.to_account_info(),
        SignMetadata {
            creator: ctx.accounts.admin.to_account_info(),
            metadata: ctx.accounts.metadata_account.to_account_info()
        }
    ))?;

    msg!("sign metadata account successfully!");

    Ok(())
}

#[derive(Accounts)]
#[instruction(decimals: u8)]
pub struct CreateToken<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        seeds = [INIT_SEED],
        bump,
        has_one = admin
    )]
    pub init: Box<Account<'info, init::Init>>,

    #[account(
        init,
        payer = admin,
        seeds = [TOKEN_SEED],
        bump,
        mint::decimals = decimals,
        mint::authority = mint,
        mint::freeze_authority = mint,
    )]
    pub mint: Box<Account<'info, Mint>>,

    /// CHECK: address
    #[account(
        mut,
        address= find_metadata_account(&mint.key()).0
    )]
    pub metadata_account: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    pub metadata_program: Program<'info, Metadata>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

pub fn mint_token(ctx: Context<MintToken>, amount: u64) -> Result<()> {
    let signer_seeds: &[&[&[u8]]] = &[&[TOKEN_SEED, &[ctx.bumps.mint]]];

    msg!("minting !");

    token::mint_to(CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.token_account_to.to_account_info(),
            authority: ctx.accounts.mint.to_account_info(),
        }, signer_seeds
    ), amount)?;

    emit!(MintTokenEvent{
        mint: ctx.accounts.mint.to_account_info().key(),
        token_account_to: ctx.accounts.token_account_to.to_account_info().key(),
        amount: amount
    });

    Ok(())
}

#[derive(Accounts)]
pub struct MintToken<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        seeds = [INIT_SEED],
        bump,
        has_one = admin
    )]
    pub init: Box<Account<'info, init::Init>>,

    /// CHECK: address
    #[account(
        mut,
        seeds = [TOKEN_SEED],
        bump,
    )]
    pub mint: Box<Account<'info, Mint>>,

    /// CHECK: address
    #[account(mut)]
    pub to: AccountInfo<'info>,

    #[account(
        init_if_needed,
        payer = admin,
        associated_token::mint = mint, // mint account
        associated_token::authority = to, // wallet account
    )]
    pub token_account_to: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

#[event]
pub struct MintTokenEvent {
    mint: Pubkey,
    token_account_to: Pubkey,
    amount: u64,
}

pub fn revoke_mint_authority(ctx: Context<RevokeMintAuthority>) -> Result<()> {
    let signer_seeds: &[&[&[u8]]] = &[&[TOKEN_SEED, &[ctx.bumps.mint]]];

    msg!("Revoking mint authority");

    token::set_authority(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            SetAuthority {
                account_or_mint: ctx.accounts.mint.to_account_info(),
                current_authority: ctx.accounts.mint.to_account_info(),
            },
            signer_seeds
        ),
        AuthorityType::MintTokens,
        None,
    )?;

    msg!("Mint authority revoked successfully");

    Ok(())
}

#[derive(Accounts)]
pub struct RevokeMintAuthority<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        seeds = [INIT_SEED],
        bump,
        has_one = admin
    )]
    pub init: Box<Account<'info, init::Init>>,

    #[account(
        mut,
        seeds = [TOKEN_SEED],
        bump,
    )]
    pub mint: Box<Account<'info, Mint>>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn burn_token(ctx: Context<BurnToken>, amount: u64) -> Result<()> {
    msg!("Burning tokens");

    token::burn(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Burn {
                mint: ctx.accounts.mint.to_account_info(),
                from: ctx.accounts.token_account.to_account_info(),
                authority: ctx.accounts.owner.to_account_info(),
            },
        ),
        amount,
    )?;

    msg!("Tokens burned successfully");

    emit!(BurnTokenEvent {
        mint: ctx.accounts.mint.to_account_info().key(),
        token_account: ctx.accounts.token_account.to_account_info().key(),
        amount: amount
    });

    Ok(())
}

#[derive(Accounts)]
pub struct BurnToken<'info> {
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [TOKEN_SEED],
        bump,
    )]
    pub mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = owner,
    )]
    pub token_account: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[event]
pub struct BurnTokenEvent {
    pub mint: Pubkey,
    pub token_account: Pubkey,
    pub amount: u64,
}















