use concordium_cis2::{Cis2Event, MetadataUrl, TokenMetadataEvent};
use concordium_std::*;

use crate::{
    errors::CustomError,
    state::State,
    types::{ContractError, ContractResult, ContractTokenAmount, ContractTokenId},
};

#[derive(SchemaType, Deserial, Serial)]
pub struct RemoveParams {
    pub tokens: Vec<ContractTokenId>,
}

#[receive(
    contract = "cis2_dsid",
    name = "remove",
    parameter = "RemoveParams",
    error = "ContractError",
    enable_logger,
    mutable
)]
/// Removes a token from the contract.
/// - This function does not fail if the token does not exist.
/// - This function fails if the token has valid balances.
/// - This function fails if the sender is not the owner of the contract.
pub fn remove<S: HasStateApi>(
    ctx: &impl HasReceiveContext,
    host: &mut impl HasHost<State<S>, StateApiType = S>,
    logger: &mut impl HasLogger,
) -> ContractResult<()> {
    // Check that the sender is the owner of the contract.
    ensure!(
        ctx.sender().matches_account(&ctx.owner()),
        ContractError::Unauthorized
    );

    let params: RemoveParams = ctx.parameter_cursor().get()?;
    let state = host.state_mut();
    for token_id in params.tokens {
        // Ensure that the token exists.
        ensure!(state.has_token(token_id), ContractError::InvalidTokenId);
        // Ensure that tokens does not have valid balances.
        ensure!(
            !state.has_balances(token_id, ctx.metadata().slot_time()),
            ContractError::Custom(CustomError::TokenHasValidBalances)
        );

        // Remove the token from the state.
        state.remove_token(token_id);

        // Log the empty token metadata.
        // This is done to ensure that the token metadata is removed from any off-chain listeners.
        logger.log(&Cis2Event::TokenMetadata::<_, ContractTokenAmount>(
            TokenMetadataEvent {
                token_id,
                metadata_url: MetadataUrl {
                    url: String::new(),
                    hash: None,
                },
            },
        ))?;
    }
    Ok(())
}

#[concordium_cfg_test]
mod tests {
    use super::*;
    use concordium_cis2::*;
    use concordium_std::test_infrastructure::*;

    const ACCOUNT_0: AccountAddress = AccountAddress([0u8; 32]);
    const ACCOUNT_1: AccountAddress = AccountAddress([1u8; 32]);
    const ADDRESS_0: Address = Address::Account(ACCOUNT_0);
    const TOKEN_0: ContractTokenId = TokenIdU8(2);
    const TOKEN_1: ContractTokenId = TokenIdU8(3);

    #[concordium_test]
    fn test_remove() {
        let mut ctx = TestReceiveContext::empty();
        ctx.set_sender(ADDRESS_0);
        ctx.set_owner(ACCOUNT_0);
        ctx.set_metadata_slot_time(Timestamp::from_timestamp_millis(99));

        let remove_token_params = RemoveParams {
            tokens: vec![TOKEN_0, TOKEN_1],
        };
        let parameter = to_bytes(&remove_token_params);
        ctx.set_parameter(&parameter);
        let mut state_builder = TestStateBuilder::new();
        let mut state = State::empty(&mut state_builder);
        // Add tokens to the state.
        state.add_token(
            &mut state_builder,
            TOKEN_0,
            MetadataUrl {
                url: "https://example.com".to_string(),
                hash: None,
            },
        );
        state.add_token(
            &mut state_builder,
            TOKEN_1,
            MetadataUrl {
                url: "https://example.com/1".to_string(),
                hash: None,
            },
        );
        // Add a balance to the token.
        // since this token is expired it should be possible to remove the token.
        claim!(state
            .mint(
                TOKEN_0,
                ACCOUNT_1,
                ContractTokenAmount::from(1),
                Timestamp::from_timestamp_millis(90),
            )
            .is_ok());
        let mut host = TestHost::new(state, state_builder);
        let mut logger = TestLogger::init();
        let result: ContractResult<()> = remove(&ctx, &mut host, &mut logger);
        assert_eq!(result, Ok(()));

        // Ensure that the tokens are removed from the state.
        assert!(!host.state().has_token(TOKEN_0));

        // Ensure that the token metadata is logged.
        assert_eq!(
            logger.logs,
            vec![
                to_bytes(&Cis2Event::TokenMetadata::<_, ContractTokenAmount>(
                    TokenMetadataEvent {
                        token_id: TOKEN_0,
                        metadata_url: MetadataUrl {
                            url: String::new(),
                            hash: None,
                        },
                    }
                )),
                to_bytes(&Cis2Event::TokenMetadata::<_, ContractTokenAmount>(
                    TokenMetadataEvent {
                        token_id: TOKEN_1,
                        metadata_url: MetadataUrl {
                            url: String::new(),
                            hash: None,
                        },
                    }
                )),
            ]
        );
    }

    #[concordium_test]
    fn test_remove_not_owner() {
        let mut ctx = TestReceiveContext::empty();
        ctx.set_sender(ADDRESS_0);
        ctx.set_owner(AccountAddress([1u8; 32]));
        ctx.set_metadata_slot_time(Timestamp::from_timestamp_millis(99));

        let remove_token_params = RemoveParams {
            tokens: vec![TOKEN_0, TOKEN_1],
        };
        let parameter = to_bytes(&remove_token_params);
        ctx.set_parameter(&parameter);
        let mut state_builder = TestStateBuilder::new();
        let mut state = State::empty(&mut state_builder);
        // Add tokens to the state.
        state.add_token(
            &mut state_builder,
            TOKEN_0,
            MetadataUrl {
                url: String::new(),
                hash: None,
            },
        );
        state.add_token(
            &mut state_builder,
            TOKEN_1,
            MetadataUrl {
                url: String::new(),
                hash: None,
            },
        );
        let mut host = TestHost::new(state, state_builder);
        let mut logger = TestLogger::init();
        let result: ContractResult<()> = remove(&ctx, &mut host, &mut logger);
        assert_eq!(result, Err(ContractError::Unauthorized));
    }

    #[concordium_test]
    fn test_remove_invalid_token_id() {
        let mut ctx = TestReceiveContext::empty();
        ctx.set_sender(ADDRESS_0);
        ctx.set_owner(ACCOUNT_0);
        ctx.set_metadata_slot_time(Timestamp::from_timestamp_millis(99));

        let remove_token_params = RemoveParams {
            tokens: vec![TOKEN_0, TOKEN_1],
        };
        let parameter = to_bytes(&remove_token_params);
        ctx.set_parameter(&parameter);
        let mut state_builder = TestStateBuilder::new();
        let mut state = State::empty(&mut state_builder);
        // Add tokens to the state.
        state.add_token(
            &mut state_builder,
            TOKEN_0,
            MetadataUrl {
                url: String::new(),
                hash: None,
            },
        );

        let mut host = TestHost::new(state, state_builder);
        let mut logger = TestLogger::init();
        let result: ContractResult<()> = remove(&ctx, &mut host, &mut logger);
        assert_eq!(result, Err(ContractError::InvalidTokenId));
    }

    #[concordium_test]
    fn test_remove_token_has_valid_balances() {
        let mut ctx = TestReceiveContext::empty();
        ctx.set_sender(ADDRESS_0);
        ctx.set_owner(ACCOUNT_0);
        ctx.set_metadata_slot_time(Timestamp::from_timestamp_millis(99));

        let remove_token_params = RemoveParams {
            tokens: vec![TOKEN_0, TOKEN_1],
        };
        let parameter = to_bytes(&remove_token_params);
        ctx.set_parameter(&parameter);
        let mut state_builder = TestStateBuilder::new();
        let mut state = State::empty(&mut state_builder);
        // Add tokens to the state.
        state.add_token(
            &mut state_builder,
            TOKEN_0,
            MetadataUrl {
                url: String::new(),
                hash: None,
            },
        );
        state.add_token(
            &mut state_builder,
            TOKEN_1,
            MetadataUrl {
                url: String::new(),
                hash: None,
            },
        );
        // Add balances to the state.
        claim!(state
            .mint(
                TOKEN_0,
                ACCOUNT_1,
                ContractTokenAmount::from(1),
                Timestamp::from_timestamp_millis(100),
            )
            .is_ok());
        let mut host = TestHost::new(state, state_builder);
        let mut logger = TestLogger::init();
        let result: ContractResult<()> = remove(&ctx, &mut host, &mut logger);
        assert_eq!(
            result,
            Err(ContractError::Custom(CustomError::TokenHasValidBalances))
        );
    }
}
