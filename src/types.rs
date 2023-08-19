use concordium_cis2::{
    BalanceOfQuery, BalanceOfQueryParams, BalanceOfQueryResponse, TokenMetadataQueryParams,
    TransferParams,
};

pub type ContractTokenId = concordium_cis2::TokenIdU8;
pub type ContractTokenAmount = concordium_cis2::TokenAmountU16;
pub type ContractError = concordium_cis2::Cis2Error<crate::errors::CustomError>;
pub type ContractEvent = concordium_cis2::Cis2Event<ContractTokenId, ContractTokenAmount>;
pub type ContractResult<T> = Result<T, ContractError>;

/// Parameter type for the CIS-2 function `balanceOf` specialized to the subset
/// of TokenIDs used by this contract.
pub type ContractBalanceOfQueryParams = BalanceOfQueryParams<ContractTokenId>;
pub type ContractExpiryOfQueryParams = BalanceOfQueryParams<ContractTokenId>;
pub type ContractExpiryOfQuery = BalanceOfQuery<ContractTokenId>;

/// Response type for the CIS-2 function `balanceOf` specialized to the subset
/// of TokenAmounts used by this contract.
pub type ContractBalanceOfQueryResponse = BalanceOfQueryResponse<ContractTokenAmount>;
/// Parameter type for the CIS-2 function `tokenMetadata` specialized to the
/// subset of TokenIDs used by this contract.
pub type ContractTokenMetadataQueryParams = TokenMetadataQueryParams<ContractTokenId>;
pub type ContractTransferParams = TransferParams<ContractTokenId, ContractTokenAmount>;
