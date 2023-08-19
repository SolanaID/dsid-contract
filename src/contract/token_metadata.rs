use concordium_cis2::{MetadataUrl, TokenMetadataQueryResponse};
use concordium_std::*;

use crate::{
    state::State,
    types::{ContractError, ContractResult, ContractTokenMetadataQueryParams},
};

#[receive(
    contract = "cis2_dsid",
    name = "tokenMetadata",
    parameter = "ContractTokenMetadataQueryParams",
    return_value = "TokenMetadataQueryResponse",
    error = "ContractError"
)]
pub fn token_metadata<S: HasStateApi>(
    ctx: &impl HasReceiveContext,
    host: &impl HasHost<State<S>, StateApiType = S>,
) -> ContractResult<TokenMetadataQueryResponse> {
    // Parse the parameter.
    let params: ContractTokenMetadataQueryParams = ctx.parameter_cursor().get()?;
    let state = host.state();
    let response: Vec<MetadataUrl> = params
        .queries
        .iter()
        .map(|q| state.get_token_metadata(q))
        .collect::<Result<Vec<MetadataUrl>, ContractError>>()?;

    Ok(TokenMetadataQueryResponse::from(response))
}

#[concordium_cfg_test]
mod tests {
    use super::*;
    use crate::types::*;
    use concordium_cis2::*;
    use concordium_std::test_infrastructure::*;

    #[concordium_test]
    fn test_token_metadata() {
        const TOKEN_0: ContractTokenId = TokenIdU8(2);
        const TOKEN_1: ContractTokenId = TokenIdU8(3);

        let mut ctx = TestReceiveContext::empty();
        let params = ContractTokenMetadataQueryParams {
            queries: vec![TOKEN_0, TOKEN_1],
        };
        let parameter = &to_bytes(&params);
        ctx.set_parameter(parameter);
        let mut state_builder = TestStateBuilder::new();
        let mut state = State::empty(&mut state_builder);

        // Add some tokens to the state.
        state.add_token(
            &mut state_builder,
            TOKEN_0,
            MetadataUrl {
                url: "https://example.com".to_string(),
                hash: Some([1; 32]),
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

        let host = TestHost::new(state, state_builder);
        let result = token_metadata(&ctx, &host).unwrap();
        assert_eq!(result.0.len(), 2);
        assert_eq!(result.0[0].url, "https://example.com");
        assert_eq!(result.0[0].hash, Some([1; 32]));
        assert_eq!(result.0[1].url, "https://example.com/1");
    }
}
