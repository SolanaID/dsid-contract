use concordium_std::*;

use crate::{errors::CustomError, state::State, types::*};

#[receive(
    contract = "cis2_dsid",
    name = "balanceOf",
    parameter = "ContractBalanceOfQueryParams",
    return_value = "ContractBalanceOfQueryResponse",
    error = "ContractError"
)]
pub fn balance_of<S: HasStateApi>(
    ctx: &impl HasReceiveContext,
    host: &impl HasHost<State<S>, StateApiType = S>,
) -> ContractResult<ContractBalanceOfQueryResponse> {
    // Parse the parameter.
    let params: ContractBalanceOfQueryParams = ctx.parameter_cursor().get()?;
    let state = host.state();
    let response: Vec<ContractTokenAmount> = params
        .queries
        .iter()
        .map(|q| match q.address {
            Address::Account(address) => {
                state.get_account_balance(q.token_id, address, ctx.metadata().slot_time())
            }
            Address::Contract(_) => Err(ContractError::Custom(CustomError::AccountsOnly)),
        })
        .collect::<Result<Vec<ContractTokenAmount>, ContractError>>()?;

    let result = ContractBalanceOfQueryResponse::from(response);
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
    fn test_balance_of() {
        let mut ctx = TestReceiveContext::empty();
        ctx.set_metadata_slot_time(Timestamp::from_timestamp_millis(150));
        let params = ContractBalanceOfQueryParams {
            queries: vec![
                BalanceOfQuery {
                    address: concordium_std::Address::Account(ACCOUNT_0),
                    token_id: TOKEN_0,
                },
                BalanceOfQuery {
                    address: concordium_std::Address::Account(ACCOUNT_0),
                    token_id: TOKEN_1,
                },
                BalanceOfQuery {
                    address: concordium_std::Address::Account(ACCOUNT_1),
                    token_id: TOKEN_0,
                },
                BalanceOfQuery {
                    address: concordium_std::Address::Account(ACCOUNT_1),
                    token_id: TOKEN_1,
                },
            ],
        };
        let parameter = &to_bytes(&params);
        ctx.set_parameter(parameter);
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
        state
            .mint(
                TOKEN_0,
                ACCOUNT_0,
                1.into(),
                Timestamp::from_timestamp_millis(100),
            )
            .expect("Failed to mint token");
        state
            .mint(
                TOKEN_1,
                ACCOUNT_0,
                1.into(),
                Timestamp::from_timestamp_millis(200),
            )
            .expect("Failed to mint token");
        state
            .mint(
                TOKEN_0,
                ACCOUNT_1,
                1.into(),
                Timestamp::from_timestamp_millis(250),
            )
            .expect("Failed to mint token");
        state
            .mint(
                TOKEN_1,
                ACCOUNT_1,
                1.into(),
                Timestamp::from_timestamp_millis(300),
            )
            .expect("Failed to mint token");

        // Check Balances
        let host = TestHost::new(state, state_builder);
        let result = balance_of(&ctx, &host);
        claim!(result.is_ok());

        // Check balance of account 0.
        let result = result.unwrap();
        claim_eq!(result.0.len(), 4);

        // Balance is `0` because the balance is expired
        claim_eq!(result.0[0], 0.into());
        claim_eq!(result.0[1], 1.into());
        claim_eq!(result.0[1], 1.into());
        claim_eq!(result.0[1], 1.into());
    }
}
