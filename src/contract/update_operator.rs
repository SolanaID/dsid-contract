use concordium_std::*;

use crate::{
    state::State,
    types::{ContractError, ContractResult},
};

#[receive(
    contract = "cis2_dsid",
    name = "updateOperator",
    parameter = "concordium_cis2::UpdateOperatorParams",
    error = "ContractError",
    mutable
)]
fn contract_update_operator<S: HasStateApi>(
    _ctx: &impl HasReceiveContext,
    _host: &mut impl HasHost<State<S>, StateApiType = S>,
) -> ContractResult<()> {
    // Update of operator is not allowed.
    Err(ContractError::Unauthorized)
}

#[concordium_cfg_test]
mod tests {
    use super::*;
    use concordium_cis2::*;
    use concordium_std::test_infrastructure::*;

    const ACCOUNT_0: AccountAddress = AccountAddress([0u8; 32]);

    #[concordium_test]
    fn test_update_operator() {
        let mut ctx = TestReceiveContext::empty();
        let update_operator_param = UpdateOperator {
            operator: Address::Account(ACCOUNT_0),
            update: OperatorUpdate::Add,
        };
        let parameter = UpdateOperatorParams(vec![update_operator_param]);
        let parameter_bytes = to_bytes(&parameter);
        ctx.set_parameter(&parameter_bytes);
        let mut state_builder = TestStateBuilder::new();
        let state = State::empty(&mut state_builder);
        let mut host = TestHost::new(state, state_builder);
        let result: ContractResult<()> = contract_update_operator(&ctx, &mut host);
        assert_eq!(result, Err(ContractError::Unauthorized));
    }
}
