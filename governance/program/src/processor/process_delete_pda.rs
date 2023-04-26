//! Program state processor

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    pubkey::Pubkey,
};

use crate::{constants::UNQ_CLUB_AUTHORITY, error::GovernanceError};

/// Processes RemoveTransaction instruction
pub fn process_delete_pda(_program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let signer = next_account_info(account_info_iter)?;
    if !signer.is_signer && signer.key != &UNQ_CLUB_AUTHORITY.parse::<Pubkey>().unwrap() {
        return Err(GovernanceError::AuthorityMissmatch.into());
    };

    let source_account_info = next_account_info(account_info_iter)?;
    let recipient_account_info = next_account_info(account_info_iter)?;

    let recipient_starting_lamports = recipient_account_info.lamports();

    **recipient_account_info.lamports.borrow_mut() = recipient_starting_lamports
        .checked_add(source_account_info.lamports())
        .unwrap();
    **source_account_info.lamports.borrow_mut() = 0;

    let mut source_data = source_account_info.data.borrow_mut();
    source_data.fill(0);

    Ok(())
}
