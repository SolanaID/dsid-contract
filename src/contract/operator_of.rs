use concordium_cis2::{OperatorOfQueryParams, OperatorOfQueryResponse};
use concordium_std::*;

use crate::{state::State, types::ContractResult};

#[receive(
    contract = "cis2_dsid",
    name = "operatorOf",
    parameter = "OperatorOfQueryParams",
    return_value = "OperatorOfQueryResponse"
)]
pub fn contract_operator_of<S: HasStateApi>(
    ctx: &impl HasReceiveContext,
    _host: &impl HasHost<State<S>, StateApiType = S>,
) -> ContractResult<OperatorOfQueryResponse> {
    let params: OperatorOfQueryParams = ctx.parameter_cursor().get()?;
    let response = params.queries.iter().map(|_| false).collect();
    Ok(OperatorOfQueryResponse(response))
}

#[concordium_cfg_test]
mod tests {
    use super::*;
    use concordium_cis2::*;
    use concordium_std::test_infrastructure::*;

    const ACCOUNT_0: AccountAddress = AccountAddress([0u8; 32]);
    const ACCOUNT_1: AccountAddress = AccountAddress([1u8; 32]);

    #[concordium_test]
    fn test_operator_of() {
        let mut ctx = TestReceiveContext::empty();
        let operator_of_param = OperatorOfQueryParams {
            queries: vec![OperatorOfQuery {
                address: Address::Account(ACCOUNT_0),
                owner: Address::Account(ACCOUNT_1),
            }],
        };
        let parameter_bytes = to_bytes(&operator_of_param);
        ctx.set_parameter(&parameter_bytes);
        let mut state_builder = TestStateBuilder::new();
        let state = State::empty(&mut state_builder);
        let host = TestHost::new(state, state_builder);
        let result: ContractResult<OperatorOfQueryResponse> = contract_operator_of(&ctx, &host);
        claim!(result.is_ok(), "Expected Ok(_), got {:?}", result);
        let response = result.unwrap();
        assert_eq!(response.0.len(), 1);
        assert!(!response.0[0]);
    }
}
