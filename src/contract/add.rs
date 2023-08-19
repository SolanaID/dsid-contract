use concordium_cis2::{Cis2Event, MetadataUrl, TokenMetadataEvent};
use concordium_std::*;

use crate::{
    state::State,
    types::{ContractError, ContractResult, ContractTokenAmount, ContractTokenId},
};

#[derive(SchemaType, Deserial, Serial)]
pub struct AddTokenParams {
    pub token_id: ContractTokenId,
    pub metadata_url: MetadataUrl,
}

#[derive(SchemaType, Deserial, Serial)]
pub struct AddParams {
    pub tokens: Vec<AddTokenParams>,
}

#[receive(
    contract = "cis2_dsid",
    name = "add",
    parameter = "AddParams",
    error = "ContractError",
    enable_logger,
    mutable
)]
/// Adds a token to the contract.
/// - This function fails if the token already exists.
/// - This function fails if the sender is not the owner of the contract.
pub fn add<S: HasStateApi>(
    ctx: &impl HasReceiveContext,
    host: &mut impl HasHost<State<S>, StateApiType = S>,
    logger: &mut impl HasLogger,
) -> ContractResult<()> {
    // Check that the sender is the owner of the contract.
    ensure!(
        ctx.sender().matches_account(&ctx.owner()),
        ContractError::Unauthorized
    );

    let params: AddParams = ctx.parameter_cursor().get()?;
    let (state, state_builder) = host.state_and_builder();
    for token in params.tokens {
        let token_id = token.token_id;
        let metadata_url = token.metadata_url;

        // Ensure that the token does not already exist.
        ensure!(!state.has_token(token_id), ContractError::InvalidTokenId);

        // Add the token to the state.
        state.add_token(state_builder, token_id, metadata_url.to_owned());

        // Log the token metadata.
        logger.log(&Cis2Event::TokenMetadata::<_, ContractTokenAmount>(
            TokenMetadataEvent {
                token_id,
                metadata_url,
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
    const ADDRESS_0: Address = Address::Account(ACCOUNT_0);
    const TOKEN_0: ContractTokenId = TokenIdU8(2);
    const TOKEN_1: ContractTokenId = TokenIdU8(3);

    #[concordium_test]
    fn test_add() {
        let mut ctx = TestReceiveContext::empty();
        ctx.set_sender(ADDRESS_0);
        ctx.set_owner(ACCOUNT_0);
        let add_token_param_0 = AddTokenParams {
            token_id: TOKEN_0,
            metadata_url: MetadataUrl {
                url: "https://example.com".to_owned(),
                hash: None,
            },
        };
        let add_token_param_1 = AddTokenParams {
            token_id: TOKEN_1,
            metadata_url: MetadataUrl {
                url: "https://example.com/1".to_owned(),
                hash: None,
            },
        };
        let add_param = AddParams {
            tokens: vec![add_token_param_0, add_token_param_1],
        };
        let parameter = to_bytes(&add_param);
        ctx.set_parameter(&parameter);
        let mut state_builder = TestStateBuilder::new();
        let state = State::empty(&mut state_builder);
        let mut host = TestHost::new(state, state_builder);
        let mut logger = TestLogger::init();
        let result: ContractResult<()> = add(&ctx, &mut host, &mut logger);
        assert_eq!(result, Ok(()));

        // Check that the token was added to the state.
        let state = host.state();
        assert!(state.has_token(TOKEN_0));
        assert!(state.has_token(TOKEN_1));

        // Check that state has token metadata.
        assert_eq!(
            state.get_token_metadata(&TOKEN_0),
            Ok(MetadataUrl {
                url: "https://example.com".to_owned(),
                hash: None,
            })
        );
        assert_eq!(
            state.get_token_metadata(&TOKEN_1),
            Ok(MetadataUrl {
                url: "https://example.com/1".to_owned(),
                hash: None,
            })
        );

        // Check that the token metadata was logged.
        let logged_events = logger.logs;
        assert_eq!(logged_events.len(), 2);
        assert_eq!(
            logged_events[0],
            to_bytes(&Cis2Event::TokenMetadata::<_, ContractTokenAmount>(
                TokenMetadataEvent {
                    token_id: TOKEN_0,
                    metadata_url: MetadataUrl {
                        url: "https://example.com".to_owned(),
                        hash: None,
                    },
                }
            ))
        );
        assert_eq!(
            logged_events[1],
            to_bytes(&Cis2Event::TokenMetadata::<_, ContractTokenAmount>(
                TokenMetadataEvent {
                    token_id: TOKEN_1,
                    metadata_url: MetadataUrl {
                        url: "https://example.com/1".to_owned(),
                        hash: None,
                    },
                }
            ))
        );
    }

    #[concordium_test]
    fn test_add_fails_if_token_already_exists() {
        let mut ctx = TestReceiveContext::empty();
        ctx.set_sender(ADDRESS_0);
        ctx.set_owner(ACCOUNT_0);
        let add_token_param_0 = AddTokenParams {
            token_id: TOKEN_0,
            metadata_url: MetadataUrl {
                url: "https://example.com".to_owned(),
                hash: None,
            },
        };
        let add_token_param_1 = AddTokenParams {
            token_id: TOKEN_0,
            metadata_url: MetadataUrl {
                url: "https://example.com/1".to_owned(),
                hash: None,
            },
        };
        let add_param = AddParams {
            tokens: vec![add_token_param_0, add_token_param_1],
        };
        let parameter = to_bytes(&add_param);
        ctx.set_parameter(&parameter);
        let mut state_builder = TestStateBuilder::new();
        let mut state = State::empty(&mut state_builder);
        state.add_token(
            &mut state_builder,
            TOKEN_0,
            MetadataUrl {
                url: "https://example.com".to_owned(),
                hash: None,
            },
        );
        let mut host = TestHost::new(state, state_builder);
        let mut logger = TestLogger::init();
        let result: ContractResult<()> = add(&ctx, &mut host, &mut logger);
        assert_eq!(result, Err(ContractError::InvalidTokenId));
    }

    #[concordium_test]
    fn test_add_fails_if_sender_is_not_owner() {
        let mut ctx = TestReceiveContext::empty();
        ctx.set_sender(ADDRESS_0);
        ctx.set_owner(AccountAddress([1u8; 32]));
        let add_token_param_0 = AddTokenParams {
            token_id: TOKEN_0,
            metadata_url: MetadataUrl {
                url: "https://example.com".to_owned(),
                hash: None,
            },
        };
        let add_token_param_1 = AddTokenParams {
            token_id: TOKEN_1,
            metadata_url: MetadataUrl {
                url: "https://example.com/1".to_owned(),
                hash: None,
            },
        };
        let add_param = AddParams {
            tokens: vec![add_token_param_0, add_token_param_1],
        };
        let parameter = to_bytes(&add_param);
        ctx.set_parameter(&parameter);
        let mut state_builder = TestStateBuilder::new();
        let state = State::empty(&mut state_builder);
        let mut host = TestHost::new(state, state_builder);
        let mut logger = TestLogger::init();
        let result: ContractResult<()> = add(&ctx, &mut host, &mut logger);
        assert_eq!(result, Err(ContractError::Unauthorized));
    }
}
