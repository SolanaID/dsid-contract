use concordium_cis2::{BurnEvent, Cis2Error, Cis2Event, MintEvent};
use concordium_std::*;

use crate::{
    errors::CustomError,
    state::State,
    types::{ContractError, ContractResult, ContractTokenAmount, ContractTokenId},
};

#[derive(Serial, Deserial, SchemaType)]
pub struct MintParam {
    /// The amount of tokens to mint.
    pub amount: ContractTokenAmount,
    /// The expiry of the minted tokens.
    pub expiry: Timestamp,
}

#[derive(Serial, Deserial, SchemaType)]
pub struct MintParams {
    /// Owner of the newly minted tokens.
    pub owner: AccountAddress,
    /// A collection of tokens to mint.
    pub tokens: collections::BTreeMap<ContractTokenId, MintParam>,
}

#[receive(
    contract = "cis2_dsid",
    name = "mint",
    parameter = "MintParams",
    error = "ContractError",
    enable_logger,
    mutable
)]
/// Mint tokens to the contract.
/// - This function fails if the sender is not the owner of the contract.
/// - This function fails if the token does not exist.
pub fn mint<S: HasStateApi>(
    ctx: &impl HasReceiveContext,
    host: &mut impl HasHost<State<S>, StateApiType = S>,
    logger: &mut impl HasLogger,
) -> ContractResult<()> {
    // Check that the sender is the owner of the contract.
    ensure!(
        ctx.sender().matches_account(&ctx.owner()),
        ContractError::Unauthorized
    );

    let params: MintParams = ctx.parameter_cursor().get()?;
    let state = host.state_mut();
    for (token_id, mint_param) in params.tokens {
        // Ensure token has not already expired
        ensure!(
            mint_param.expiry > ctx.metadata().slot_time(),
            Cis2Error::Custom(CustomError::TokenExpired)
        );
        // Mint the tokens.
        let existing_balance =
            state.mint(token_id, params.owner, mint_param.amount, mint_param.expiry)?;

        if let Some(balance) = existing_balance {
            // There was an existing balance
            let amount = balance.get_balance(ctx.metadata().slot_time());
            if amount > ContractTokenAmount::from(0) {
                // The existing balances has a valid amount.
                // Log the burned tokens.
                logger.log(&Cis2Event::Burn::<_, ContractTokenAmount>(BurnEvent {
                    token_id,
                    owner: Address::Account(params.owner),
                    amount,
                }))?;
            }
        }

        // Log the minted tokens.
        logger.log(&Cis2Event::Mint::<_, ContractTokenAmount>(MintEvent {
            token_id,
            owner: Address::Account(params.owner),
            amount: mint_param.amount,
        }))?;
    }

    Ok(())
}

#[concordium_cfg_test]
mod tests {
    use super::*;
    use concordium_cis2::*;
    use concordium_std::test_infrastructure::*;

    const ACCOUNT_0: AccountAddress = AccountAddress([0u8; 32]);
    const ACCOUNT_2: AccountAddress = AccountAddress([1u8; 32]);
    const ADDRESS_0: Address = Address::Account(ACCOUNT_0);
    const TOKEN_0: ContractTokenId = TokenIdU8(0);
    const TOKEN_1: ContractTokenId = TokenIdU8(1);

    #[concordium_test]
    fn test_mint() {
        let mut ctx = TestReceiveContext::empty();
        ctx.set_sender(ADDRESS_0);
        ctx.set_owner(ACCOUNT_0);
        ctx.set_metadata_slot_time(Timestamp::from_timestamp_millis(99));

        let mint_params = MintParams {
            owner: ACCOUNT_2,
            tokens: collections::BTreeMap::from_iter(vec![
                (
                    TOKEN_0,
                    MintParam {
                        amount: ContractTokenAmount::from(100),
                        expiry: Timestamp::from_timestamp_millis(100),
                    },
                ),
                (
                    TOKEN_1,
                    MintParam {
                        amount: ContractTokenAmount::from(200),
                        expiry: Timestamp::from_timestamp_millis(200),
                    },
                ),
            ]),
        };
        let parameter_bytes = to_bytes(&mint_params);
        ctx.set_parameter(&parameter_bytes);
        let mut state_builder = TestStateBuilder::new();
        let mut state = State::empty(&mut state_builder);
        // Add the tokens to the state.
        state.add_token(
            &mut state_builder,
            TOKEN_0,
            MetadataUrl {
                url: "https://example.com".to_string(),
                hash: Option::None,
            },
        );
        state.add_token(
            &mut state_builder,
            TOKEN_1,
            MetadataUrl {
                url: "https://example.com/1".to_string(),
                hash: Option::None,
            },
        );
        let mut host = TestHost::new(state, state_builder);
        let mut logger = TestLogger::init();
        let result: ContractResult<()> = mint(&ctx, &mut host, &mut logger);

        assert!(result.is_ok());

        // Check that the tokens were minted.
        let state = host.state();
        let token_0_balance =
            state.get_account_balance(TOKEN_0, ACCOUNT_2, Timestamp::from_timestamp_millis(150));
        // The Balance is 0 because the tokens have expired.
        assert_eq!(token_0_balance, Ok(ContractTokenAmount::from(0)));

        let token_1_balance =
            state.get_account_balance(TOKEN_1, ACCOUNT_2, Timestamp::from_timestamp_millis(150));
        // The Balance is 200 because the tokens have not expired.
        assert_eq!(token_1_balance, Ok(ContractTokenAmount::from(200)));

        let events = logger.logs;
        assert_eq!(events.len(), 2);
        assert_eq!(
            events[0],
            to_bytes(&Cis2Event::Mint::<_, ContractTokenAmount>(MintEvent {
                token_id: TOKEN_0,
                owner: Address::Account(ACCOUNT_2),
                amount: ContractTokenAmount::from(100),
            }))
        );
        assert_eq!(
            events[1],
            to_bytes(&Cis2Event::Mint::<_, ContractTokenAmount>(MintEvent {
                token_id: TOKEN_1,
                owner: Address::Account(ACCOUNT_2),
                amount: ContractTokenAmount::from(200),
            }))
        );
    }

    #[concordium_test]
    fn test_mint_expired() {
        let mut ctx = TestReceiveContext::empty();
        ctx.set_sender(ADDRESS_0);
        ctx.set_owner(ACCOUNT_0);
        ctx.set_metadata_slot_time(Timestamp::from_timestamp_millis(99));

        let mint_params = MintParams {
            owner: ACCOUNT_2,
            tokens: collections::BTreeMap::from_iter(vec![(
                TOKEN_0,
                MintParam {
                    amount: ContractTokenAmount::from(100),
                    expiry: Timestamp::from_timestamp_millis(50),
                },
            )]),
        };
        let parameter_bytes = to_bytes(&mint_params);
        ctx.set_parameter(&parameter_bytes);
        let mut state_builder = TestStateBuilder::new();
        let mut state = State::empty(&mut state_builder);
        // Add the tokens to the state.
        state.add_token(
            &mut state_builder,
            TOKEN_0,
            MetadataUrl {
                url: "https://example.com".to_string(),
                hash: Option::None,
            },
        );
        let mut host = TestHost::new(state, state_builder);
        let mut logger = TestLogger::init();
        let result: ContractResult<()> = mint(&ctx, &mut host, &mut logger);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            ContractError::Custom(CustomError::TokenExpired)
        );
    }

    #[concordium_test]
    fn test_mint_no_token() {
        let mut ctx = TestReceiveContext::empty();
        ctx.set_sender(ADDRESS_0);
        ctx.set_owner(ACCOUNT_0);
        ctx.set_metadata_slot_time(Timestamp::from_timestamp_millis(99));

        let mint_params = MintParams {
            owner: ACCOUNT_2,
            tokens: collections::BTreeMap::from_iter(vec![(
                TOKEN_0,
                MintParam {
                    amount: ContractTokenAmount::from(100),
                    expiry: Timestamp::from_timestamp_millis(100),
                },
            )]),
        };
        let parameter_bytes = to_bytes(&mint_params);
        ctx.set_parameter(&parameter_bytes);
        let mut state_builder = TestStateBuilder::new();
        let state = State::empty(&mut state_builder);
        let mut host = TestHost::new(state, state_builder);
        let mut logger = TestLogger::init();
        let result: ContractResult<()> = mint(&ctx, &mut host, &mut logger);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ContractError::InvalidTokenId);
    }

    #[concordium_test]
    fn test_mint_unauthorized() {
        let mut ctx = TestReceiveContext::empty();
        ctx.set_sender(ADDRESS_0);
        ctx.set_owner(ACCOUNT_2);
        ctx.set_metadata_slot_time(Timestamp::from_timestamp_millis(99));

        let mint_params = MintParams {
            owner: ACCOUNT_2,
            tokens: collections::BTreeMap::from_iter(vec![(
                TOKEN_0,
                MintParam {
                    amount: ContractTokenAmount::from(100),
                    expiry: Timestamp::from_timestamp_millis(100),
                },
            )]),
        };
        let parameter_bytes = to_bytes(&mint_params);
        ctx.set_parameter(&parameter_bytes);
        let mut state_builder = TestStateBuilder::new();
        let mut state = State::empty(&mut state_builder);
        // Add the tokens to the state.
        state.add_token(
            &mut state_builder,
            TOKEN_0,
            MetadataUrl {
                url: "https://example.com".to_string(),
                hash: Option::None,
            },
        );
        let mut host = TestHost::new(state, state_builder);
        let mut logger = TestLogger::init();
        let result: ContractResult<()> = mint(&ctx, &mut host, &mut logger);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ContractError::Unauthorized);
    }

    #[concordium_test]
    fn test_burn_existing_token() {
        let mut ctx = TestReceiveContext::empty();
        ctx.set_sender(ADDRESS_0);
        ctx.set_owner(ACCOUNT_0);
        ctx.set_metadata_slot_time(Timestamp::from_timestamp_millis(50));

        let mint_params = MintParams {
            owner: ACCOUNT_2,
            tokens: collections::BTreeMap::from_iter(vec![
                (
                    TOKEN_0,
                    MintParam {
                        amount: ContractTokenAmount::from(100),
                        expiry: Timestamp::from_timestamp_millis(100),
                    },
                ),
                (
                    TOKEN_1,
                    MintParam {
                        amount: ContractTokenAmount::from(200),
                        expiry: Timestamp::from_timestamp_millis(200),
                    },
                ),
            ]),
        };
        let parameter_bytes = to_bytes(&mint_params);
        ctx.set_parameter(&parameter_bytes);
        let mut state_builder = TestStateBuilder::new();
        let mut state = State::empty(&mut state_builder);
        // Add the tokens to the state.
        state.add_token(
            &mut state_builder,
            TOKEN_0,
            MetadataUrl {
                url: "https://example.com".to_string(),
                hash: Option::None,
            },
        );
        state.add_token(
            &mut state_builder,
            TOKEN_1,
            MetadataUrl {
                url: "https://example.com/1".to_string(),
                hash: Option::None,
            },
        );

        // Add token balances to the state
        claim!(state
            .mint(
                TOKEN_0,
                ACCOUNT_2,
                ContractTokenAmount::from(10),
                Timestamp::from_timestamp_millis(90),
            )
            .is_ok());
        claim!(state
            .mint(
                TOKEN_1,
                ACCOUNT_2,
                ContractTokenAmount::from(20),
                Timestamp::from_timestamp_millis(30),
            )
            .is_ok());

        let mut host = TestHost::new(state, state_builder);
        let mut logger = TestLogger::init();
        let result: ContractResult<()> = mint(&ctx, &mut host, &mut logger);

        assert!(result.is_ok());
        let events = logger.logs;
        assert_eq!(events.len(), 3);
        assert_eq!(
            events[0],
            to_bytes(&Cis2Event::Burn::<_, ContractTokenAmount>(BurnEvent {
                token_id: TOKEN_0,
                owner: Address::Account(ACCOUNT_2),
                amount: ContractTokenAmount::from(10),
            }))
        );
        assert_eq!(
            events[1],
            to_bytes(&Cis2Event::Mint::<_, ContractTokenAmount>(MintEvent {
                token_id: TOKEN_0,
                owner: Address::Account(ACCOUNT_2),
                amount: ContractTokenAmount::from(100),
            }))
        );
        assert_eq!(
            events[2],
            to_bytes(&Cis2Event::Mint::<_, ContractTokenAmount>(MintEvent {
                token_id: TOKEN_1,
                owner: Address::Account(ACCOUNT_2),
                amount: ContractTokenAmount::from(200),
            }))
        );
    }
}
