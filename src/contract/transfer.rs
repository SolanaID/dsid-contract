use concordium_std::*;

use crate::{
    state::State,
    types::{ContractError, ContractResult},
};

#[receive(
    contract = "cis2_dsid",
    name = "transfer",
    parameter = "crate::types::ContractTransferParams",
    error = "ContractError",
    mutable
)]
pub fn transfer<S: HasStateApi>(
    _ctx: &impl HasReceiveContext,
    _host: &mut impl HasHost<State<S>, StateApiType = S>,
) -> ContractResult<()> {
    // Transfer of tokens is not allowed.
    Err(ContractError::Unauthorized)
}

#[concordium_cfg_test]
mod tests {
    use super::*;
    use crate::types::*;
    use concordium_cis2::*;
    use concordium_std::test_infrastructure::*;

    const ACCOUNT_0: AccountAddress = AccountAddress([0u8; 32]);
    const ADDRESS_0: Address = Address::Account(ACCOUNT_0);
    const ACCOUNT_1: AccountAddress = AccountAddress([1u8; 32]);
    const TOKEN_0: ContractTokenId = TokenIdU8(2);

    #[concordium_test]
    fn test_transfer() {
        let mut ctx = TestReceiveContext::empty();
        let transfer_param = concordium_cis2::Transfer {
            token_id: TOKEN_0,
            amount: crate::types::ContractTokenAmount::from(100),
            from: ADDRESS_0,
            to: Receiver::from_account(ACCOUNT_1),
            data: AdditionalData::empty(),
        };
        let parameter = ContractTransferParams::from(vec![transfer_param]);
        let parameter_bytes = to_bytes(&parameter);
        ctx.set_parameter(&parameter_bytes);
        let mut state_builder = TestStateBuilder::new();
        let state = State::empty(&mut state_builder);
        let mut host = TestHost::new(state, state_builder);
        let result: ContractResult<()> = transfer(&ctx, &mut host);
        assert_eq!(result, Err(ContractError::Unauthorized));
    }
}
