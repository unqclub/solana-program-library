//! Program state processor

use crate::{
    error::GovernanceError,
    state::{
        enums::GovernanceAccountType,
        governance::{
            assert_valid_create_governance_args, get_governance_address_seeds, GovernanceConfig,
            GovernanceV2,
        },
        realm::get_realm_data,
        token_owner_record::get_token_owner_record_data_for_realm,
    },
};
use delegation_manager::check_authorization;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};

use spl_governance_tools::account::create_and_serialize_account_signed;

/// Processes CreateGovernance instruction
pub fn process_create_governance(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    config: GovernanceConfig,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let realm_info = next_account_info(account_info_iter)?; // 0
    let governance_info = next_account_info(account_info_iter)?; // 1
    let governed_account_info = next_account_info(account_info_iter)?; // 2

    let token_owner_record_info = next_account_info(account_info_iter)?; // 3

    let payer_info = next_account_info(account_info_iter)?; // 4
    let system_info = next_account_info(account_info_iter)?; // 5

    let rent = Rent::get()?;

    let create_authority_info = next_account_info(account_info_iter)?; // 6

    assert_valid_create_governance_args(program_id, &config, realm_info)?;

    let realm_data = get_realm_data(program_id, realm_info)?;

    realm_data.assert_create_authority_can_create_governance(
        program_id,
        realm_info.key,
        token_owner_record_info,
        create_authority_info,
        account_info_iter, // realm_config_info 7, voter_weight_record_info 8
    )?;

    let token_owner_record_data =
        get_token_owner_record_data_for_realm(program_id, token_owner_record_info, realm_info.key)?;

    // Proposal owner (TokenOwner) or its governance_delegate or representative must sign this transaction
    if let Some(delegation_info) = account_info_iter.next() {
        check_authorization(create_authority_info, payer_info, Some(delegation_info))?;
        if !payer_info.is_signer {
            return Err(GovernanceError::RepresentativeMustSign.into());
        }
    } else {
        if realm_data.authority == Some(*create_authority_info.key) {
            if !create_authority_info.is_signer {
                return Err(GovernanceError::RealmAuthorityMustSign.into());
            }
        }
        token_owner_record_data.assert_token_owner_or_delegate_is_signer(create_authority_info)?;
    }

    let governance_data = GovernanceV2 {
        account_type: GovernanceAccountType::GovernanceV2,
        realm: *realm_info.key,
        governed_account: *governed_account_info.key,
        config,
        proposals_count: 0,
        reserved: [0; 6],
        voting_proposal_count: 0,
        reserved_v2: [0; 128],
    };

    create_and_serialize_account_signed::<GovernanceV2>(
        payer_info,
        governance_info,
        &governance_data,
        &get_governance_address_seeds(realm_info.key, governed_account_info.key),
        program_id,
        system_info,
        &rent,
    )?;

    Ok(())
}
