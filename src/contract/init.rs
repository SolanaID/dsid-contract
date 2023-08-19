use concordium_std::*;

use crate::state::State;

/// Initialize contract instance with a no token types.
#[init(contract = "cis2_dsid", event = "crate::types::ContractEvent")]
pub fn init<S: HasStateApi>(
    _ctx: &impl HasInitContext,
    state_builder: &mut StateBuilder<S>,
) -> InitResult<State<S>> {
    // Construct the initial contract state.
    Ok(State::empty(state_builder))
}

#[concordium_cfg_test]
mod tests {
    use super::*;
    use concordium_std::test_infrastructure::*;

    #[concordium_test]
    fn test_init() {
        let ctx = TestInitContext::empty();
        let mut state_builder = TestStateBuilder::new();
        let result: InitResult<State<TestStateApi>> = init(&ctx, &mut state_builder);
        claim!(result.is_ok(), "Expected Ok");
    }
}
