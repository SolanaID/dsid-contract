pub mod add;
pub mod balance_of;
pub mod expiry_of;
pub mod init;
pub mod mint;
pub mod operator_of;
pub mod remove;
pub mod token_metadata;
pub mod transfer;
pub mod update_operator;
use concordium_std::concordium_cfg_test;

#[concordium_cfg_test]
mod tests {
    use crate::contract::{
        add::*, balance_of::*, expiry_of::*, init::*, mint::*, remove::*, token_metadata::*,
    };
    use crate::state::*;
    use crate::types::*;
    use concordium_cis2::*;
    use concordium_std::test_infrastructure::*;
    use concordium_std::*;

    // Contract Owner
    const ACCOUNT_OWNER: AccountAddress = AccountAddress([0u8; 32]);
    const ADDRESS_OWNER: Address = Address::Account(ACCOUNT_OWNER);
    const ACCOUNT_1: AccountAddress = AccountAddress([1u8; 32]);
    const ACCOUNT_2: AccountAddress = AccountAddress([2u8; 32]);
    const TOKEN_0: ContractTokenId = TokenIdU8(2);
    const TOKEN_1: ContractTokenId = TokenIdU8(3);

    #[concordium_test]
    fn test_complete_flow() {
        // This tests the complete flow of the contract.
        // It is not a unit test, but rather an integration test.
        // It is not meant to be run on the CI, but rather locally.

        // Initialize the contract.
        let init_ctx = TestInitContext::empty();
        let mut state_builder = TestStateBuilder::new();
        let init_result: InitResult<State<TestStateApi>> = init(&init_ctx, &mut state_builder);
        claim!(init_result.is_ok(), "Expected Ok");

        let state = init_result.unwrap();
        let mut host = TestHost::new(state, state_builder);
        let now = Timestamp::from_timestamp_millis(50);

        // Add a token.
        let mut add_ctx = TestReceiveContext::empty();
        add_ctx.set_sender(ADDRESS_OWNER);
        add_ctx.set_owner(ACCOUNT_OWNER);
        add_ctx.set_metadata_slot_time(now);

        let params = AddParams {
            tokens: vec![
                AddTokenParams {
                    token_id: TOKEN_0,
                    metadata_url: MetadataUrl {
                        url: "https://example.com".to_string(),
                        hash: None,
                    },
                },
                AddTokenParams {
                    token_id: TOKEN_1,
                    metadata_url: MetadataUrl {
                        url: "https://example.com/1".to_string(),
                        hash: None,
                    },
                },
            ],
        };
        let add_parameter = &to_bytes(&params);
        add_ctx.set_parameter(add_parameter);
        let mut logger = TestLogger::init();
        let add_result: ContractResult<()> = add(&add_ctx, &mut host, &mut logger);
        claim!(add_result.is_ok(), "Expected Ok");

        // Check token metadata.
        let mut token_metadata_ctx = TestReceiveContext::empty();
        let token_metadata_param = ContractTokenMetadataQueryParams {
            queries: vec![TOKEN_0, TOKEN_1],
        };
        let token_metadata_parameter = &to_bytes(&token_metadata_param);
        token_metadata_ctx.set_parameter(token_metadata_parameter);
        let token_metadata_result: ContractResult<TokenMetadataQueryResponse> =
            token_metadata(&token_metadata_ctx, &host);
        claim!(token_metadata_result.is_ok(), "Expected Ok");
        let token_metadata_response = token_metadata_result.unwrap();
        claim_eq!(token_metadata_response.0.len(), 2, "Expected two tokens");
        claim_eq!(
            token_metadata_response.0[0].url,
            "https://example.com",
            "Expected url to be https://example.com"
        );
        claim_eq!(
            token_metadata_response.0[1].url,
            "https://example.com/1",
            "Expected url to be https://example.com/1"
        );

        // Mint tokens.
        let mut mint_ctx = TestReceiveContext::empty();
        mint_ctx.set_sender(ADDRESS_OWNER);
        mint_ctx.set_owner(ACCOUNT_OWNER);
        mint_ctx.set_metadata_slot_time(now);

        let mint_params = MintParams {
            owner: ACCOUNT_1,
            tokens: collections::BTreeMap::from_iter(vec![
                (
                    TOKEN_0,
                    MintParam {
                        amount: 100.into(),
                        expiry: Timestamp::from_timestamp_millis(100),
                    },
                ),
                (
                    TOKEN_1,
                    MintParam {
                        amount: 200.into(),
                        expiry: Timestamp::from_timestamp_millis(200),
                    },
                ),
            ]),
        };
        let mint_parameter = &to_bytes(&mint_params);
        mint_ctx.set_parameter(mint_parameter);
        let mut mint_logger = TestLogger::init();
        let mint_result = mint(&mint_ctx, &mut host, &mut mint_logger);
        claim!(mint_result.is_ok(), "Expected Ok");

        // Check balances.
        let mut balance_of_ctx = TestReceiveContext::empty();
        balance_of_ctx.set_metadata_slot_time(now);

        let balance_of_params = ContractBalanceOfQueryParams {
            queries: vec![
                BalanceOfQuery {
                    token_id: TOKEN_0,
                    address: Address::Account(ACCOUNT_1),
                },
                BalanceOfQuery {
                    token_id: TOKEN_1,
                    address: Address::Account(ACCOUNT_1),
                },
                BalanceOfQuery {
                    token_id: TOKEN_0,
                    address: Address::Account(ACCOUNT_2),
                },
                BalanceOfQuery {
                    token_id: TOKEN_1,
                    address: Address::Account(ACCOUNT_2),
                },
            ],
        };
        let balance_of_parameter = &to_bytes(&balance_of_params);
        balance_of_ctx.set_parameter(balance_of_parameter);
        let balance_of_result: ContractResult<ContractBalanceOfQueryResponse> =
            balance_of(&balance_of_ctx, &host);
        claim!(balance_of_result.is_ok(), "Expected Ok");
        let balance_of_response = balance_of_result.unwrap();
        claim_eq!(
            balance_of_response.0.len(),
            4,
            "Expected four balance queries"
        );
        claim_eq!(
            balance_of_response.0[0],
            100.into(),
            "Expected balance to be 100"
        );
        claim_eq!(
            balance_of_response.0[1],
            200.into(),
            "Expected balance to be 200"
        );
        claim_eq!(
            balance_of_response.0[2],
            0.into(),
            "Expected balance to be 0"
        );
        claim_eq!(
            balance_of_response.0[3],
            0.into(),
            "Expected balance to be 0"
        );

        // Check Expiry.
        let mut expiry_ctx = TestReceiveContext::empty();
        let expiry_params = ContractExpiryOfQueryParams {
            queries: vec![
                ContractExpiryOfQuery {
                    token_id: TOKEN_0,
                    address: Address::Account(ACCOUNT_1),
                },
                ContractExpiryOfQuery {
                    token_id: TOKEN_1,
                    address: Address::Account(ACCOUNT_1),
                },
                ContractExpiryOfQuery {
                    token_id: TOKEN_0,
                    address: Address::Account(ACCOUNT_2),
                },
                ContractExpiryOfQuery {
                    token_id: TOKEN_1,
                    address: Address::Account(ACCOUNT_2),
                },
            ],
        };
        let expiry_parameter = &to_bytes(&expiry_params);
        expiry_ctx.set_parameter(expiry_parameter);
        let expiry_result: ContractResult<ExpiryOfQueryResponse> = expiry_of(&expiry_ctx, &host);
        claim!(expiry_result.is_ok(), "Expected Ok");
        let expiry_response = expiry_result.unwrap();
        claim_eq!(expiry_response.0.len(), 4, "Expected four expiry queries");
        claim_eq!(
            expiry_response.0[0],
            Option::Some(Timestamp::from_timestamp_millis(100)),
            "Expected expiry to be 100"
        );
        claim_eq!(
            expiry_response.0[1],
            Option::Some(Timestamp::from_timestamp_millis(200)),
            "Expected expiry to be 200"
        );
        claim_eq!(
            expiry_response.0[2],
            Option::None,
            "Expected expiry to be None"
        );
        claim_eq!(
            expiry_response.0[3],
            Option::None,
            "Expected expiry to be None"
        );

        // After some time has passed
        let now = Timestamp::from_timestamp_millis(60);

        // Mint again to replace the existing reputation / token for ACCOUNT_1.
        let mut mint_ctx = TestReceiveContext::empty();
        mint_ctx.set_sender(ADDRESS_OWNER);
        mint_ctx.set_owner(ACCOUNT_OWNER);
        mint_ctx.set_metadata_slot_time(now);
        let mint_params = MintParams {
            owner: ACCOUNT_1,
            tokens: collections::BTreeMap::from_iter(vec![(
                TOKEN_0,
                MintParam {
                    amount: 200.into(),
                    expiry: Timestamp::from_timestamp_millis(300),
                },
            )]),
        };
        let mint_parameter = &to_bytes(&mint_params);
        mint_ctx.set_parameter(mint_parameter);
        let mut mint_logger = TestLogger::init();
        let mint_result = mint(&mint_ctx, &mut host, &mut mint_logger);
        claim!(mint_result.is_ok(), "Expected Ok");

        // Check that the balance has been updated.
        let mut balance_of_ctx = TestReceiveContext::empty();
        balance_of_ctx.set_metadata_slot_time(now);
        let balance_of_params = ContractBalanceOfQueryParams {
            queries: vec![BalanceOfQuery {
                token_id: TOKEN_0,
                address: Address::Account(ACCOUNT_1),
            }],
        };
        let balance_of_parameter = &to_bytes(&balance_of_params);
        balance_of_ctx.set_parameter(balance_of_parameter);
        let balance_of_result: ContractResult<ContractBalanceOfQueryResponse> =
            balance_of(&balance_of_ctx, &host);
        claim!(balance_of_result.is_ok(), "Expected Ok");
        let balance_of_response = balance_of_result.unwrap();
        claim_eq!(balance_of_response.0.len(), 1, "Expected one balance query");
        claim_eq!(
            balance_of_response.0[0],
            200.into(),
            "Expected balance to be 200"
        );

        // Check that the expiry has been updated.
        let mut expiry_ctx = TestReceiveContext::empty();
        let expiry_params = ContractExpiryOfQueryParams {
            queries: vec![ContractExpiryOfQuery {
                token_id: TOKEN_0,
                address: Address::Account(ACCOUNT_1),
            }],
        };
        let expiry_parameter = &to_bytes(&expiry_params);
        expiry_ctx.set_parameter(expiry_parameter);
        let expiry_result: ContractResult<ExpiryOfQueryResponse> = expiry_of(&expiry_ctx, &host);
        claim!(expiry_result.is_ok(), "Expected Ok");
        let expiry_response = expiry_result.unwrap();
        claim_eq!(expiry_response.0.len(), 1, "Expected one expiry query");
        claim_eq!(
            expiry_response.0[0],
            Option::Some(Timestamp::from_timestamp_millis(300)),
            "Expected expiry to be 300"
        );

        // Assert that Token 1 cannot be removed.
        let mut remove_ctx = TestReceiveContext::empty();
        remove_ctx.set_sender(ADDRESS_OWNER);
        remove_ctx.set_owner(ACCOUNT_OWNER);
        remove_ctx.set_metadata_slot_time(now);
        let remove_params = RemoveParams {
            tokens: vec![TOKEN_1],
        };
        let remove_parameter = &to_bytes(&remove_params);
        remove_ctx.set_parameter(remove_parameter);
        let mut remove_logger = TestLogger::init();
        let remove_result = remove(&remove_ctx, &mut host, &mut remove_logger);
        claim!(remove_result.is_err(), "Expected Err");

        // After some time has passed
        let now = Timestamp::from_timestamp_millis(1000);

        // Assert that Token 1 can be removed.
        let mut remove_ctx = TestReceiveContext::empty();
        remove_ctx.set_sender(ADDRESS_OWNER);
        remove_ctx.set_owner(ACCOUNT_OWNER);
        remove_ctx.set_metadata_slot_time(now);
        let remove_params = RemoveParams {
            tokens: vec![TOKEN_1],
        };
        let remove_parameter = &to_bytes(&remove_params);
        remove_ctx.set_parameter(remove_parameter);
        let mut remove_logger = TestLogger::init();
        let remove_result = remove(&remove_ctx, &mut host, &mut remove_logger);
        claim!(remove_result.is_ok(), "Expected Ok");
    }
}
