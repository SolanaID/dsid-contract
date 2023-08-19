use concordium_std::*;

use crate::{errors::CustomError, state::State, types::*};

#[derive(Debug, Serialize, SchemaType)]
pub struct ExpiryOfQueryResponse(#[concordium(size_length = 2)] pub Vec<Option<Timestamp>>);

#[receive(
    contract = "cis2_dsid",
    name = "expiryOf",
    parameter = "ContractExpiryOfQueryParams",
    return_value = "ExpiryOfQueryResponse",
    error = "ContractError"
)]
pub fn expiry_of<S: HasStateApi>(
    ctx: &impl HasReceiveContext,
    host: &impl HasHost<State<S>, StateApiType = S>,
) -> ContractResult<ExpiryOfQueryResponse> {
    // Parse the parameter.
    let params: ContractExpiryOfQueryParams = ctx.parameter_cursor().get()?;
    let state = host.state();
    let response: Vec<Option<Timestamp>> = params
        .queries
        .iter()
        .map(|q| match q.address {
            Address::Account(address) => state.get_account_balance_expiry(q.token_id, address),
            Address::Contract(_) => Err(ContractError::Custom(CustomError::AccountsOnly)),
        })
        .collect::<Result<Vec<Option<Timestamp>>, ContractError>>()?;

    let result = ExpiryOfQueryResponse(response);
    Ok(result)
}

#[concordium_cfg_test]
mod tests {
    use super::*;
    use concordium_cis2::*;
    use concordium_std::test_infrastructure::*;

    const ACCOUNT_0: AccountAddress = AccountAddress([0u8; 32]);
    const ACCOUNT_1: AccountAddress = AccountAddress([1u8; 32]);
    const TOKEN_0: ContractTokenId = TokenIdU8(2);
    const TOKEN_1: ContractTokenId = TokenIdU8(3);

    #[concordium_test]
    fn test_expiry_of() {
        let mut ctx = TestReceiveContext::empty();
        ctx.set_metadata_slot_time(Timestamp::from_timestamp_millis(150));
        let params = ContractExpiryOfQueryParams {
            queries: vec![
                ContractExpiryOfQuery {
                    address: concordium_std::Address::Account(ACCOUNT_0),
                    token_id: TOKEN_0,
                },
                ContractExpiryOfQuery {
                    address: concordium_std::Address::Account(ACCOUNT_0),
                    token_id: TOKEN_1,
                },
                ContractExpiryOfQuery {
                    address: concordium_std::Address::Account(ACCOUNT_1),
                    token_id: TOKEN_0,
                },
                ContractExpiryOfQuery {
                    address: concordium_std::Address::Account(ACCOUNT_1),
                    token_id: TOKEN_1,
                },
            ],
        };
        let parameter = &to_bytes(&params);
        ctx.set_parameter(parameter);

        let mut state_builder = TestStateBuilder::new();
        let mut state = State::empty(&mut state_builder);
        // Add tokens to the state
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

        // Add Account balances to the state
        state
            .mint(
                TOKEN_0,
                ACCOUNT_0,
                10.into(),
                Timestamp::from_timestamp_millis(100),
            )
            .unwrap();
        state
            .mint(
                TOKEN_1,
                ACCOUNT_0,
                20.into(),
                Timestamp::from_timestamp_millis(200),
            )
            .unwrap();
        state
            .mint(
                TOKEN_0,
                ACCOUNT_1,
                30.into(),
                Timestamp::from_timestamp_millis(300),
            )
            .unwrap();

        let host = TestHost::new(state, state_builder);
        let result = expiry_of(&ctx, &host).unwrap();
        assert_eq!(
            result.0,
            vec![
                Some(Timestamp::from_timestamp_millis(100)),
                Some(Timestamp::from_timestamp_millis(200)),
                Some(Timestamp::from_timestamp_millis(300)),
                None,
            ]
        );
    }
}
