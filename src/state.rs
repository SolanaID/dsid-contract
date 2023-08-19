use concordium_cis2::MetadataUrl;
use concordium_std::*;

use crate::types::{ContractError, ContractResult, ContractTokenAmount, ContractTokenId};

#[derive(Serial, Deserial)]
pub struct TokenBalanceState {
    pub amount: ContractTokenAmount,
    pub expiry: Timestamp,
}

impl TokenBalanceState {
    /// Checks if the token has a balance at the given time.
    pub fn has_balance(&self, now: Timestamp) -> bool {
        let balance = self.get_balance(now);
        balance > ContractTokenAmount::from(0)
    }

    /// Gets the balance of the token.
    /// - If the balance has expired, the balance is 0.
    pub fn get_balance(&self, now: Timestamp) -> ContractTokenAmount {
        if self.expiry > now {
            self.amount
        } else {
            ContractTokenAmount::from(0)
        }
    }
}

#[derive(Serial, DeserialWithState, Deletable)]
#[concordium(state_parameter = "S")]
pub struct TokenState<S> {
    balances: StateMap<AccountAddress, TokenBalanceState, S>,
    metadata: MetadataUrl,
}

impl<S> TokenState<S>
where
    S: HasStateApi,
{
    /// Gets Account Balance for a given token and account.
    /// - If the state has no entry for the given account and token, the balance is 0.
    /// - If the balance has expired, the balance is 0.
    pub(crate) fn get_account_balance(
        &self,
        account: AccountAddress,
        now: Timestamp,
    ) -> ContractTokenAmount {
        self.balances
            .get(&account)
            .map_or(ContractTokenAmount::from(0), |balance| {
                balance.get_balance(now)
            })
    }

    /// Get Account Balance Expiry for a given token and account.
    /// - If the state has no entry for the given account and token, the expiry is None.
    pub(crate) fn get_account_balance_expiry(&self, account: AccountAddress) -> Option<Timestamp> {
        self.balances.get(&account).map(|balance| balance.expiry)
    }
}

#[derive(Serial, DeserialWithState, StateClone)]
#[concordium(state_parameter = "S")]
pub struct State<S> {
    tokens: StateMap<ContractTokenId, TokenState<S>, S>,
}
impl<S> State<S>
where
    S: HasStateApi,
    S: Clone,
{
    pub(crate) fn empty(state_builder: &mut StateBuilder<S>) -> Self {
        Self {
            tokens: state_builder.new_map(),
        }
    }

    /// Checks if a token exists.
    pub(crate) fn has_token(&self, token_id: ContractTokenId) -> bool {
        self.tokens.get(&token_id).is_some()
    }

    /// Adds a token to the state.
    /// - This function does not replace an existing token.
    pub(crate) fn add_token(
        &mut self,
        state_builder: &mut StateBuilder<S>,
        token_id: ContractTokenId,
        token_metadata: MetadataUrl,
    ) {
        // Add the token to the state.
        // This is safe because it does not overwrite an existing token.
        self.tokens.entry(token_id).or_insert(TokenState {
            balances: state_builder.new_map(),
            metadata: token_metadata,
        });
    }

    /// Removes a token from the state.
    /// - This function does not fail if the token does not exist.
    pub(crate) fn remove_token(&mut self, token_id: ContractTokenId) {
        self.tokens.remove(&token_id);
    }

    /// Checks if a token has valid balances.
    /// - A tokens has valid balances if there is a balance > 0 which has not expired.
    pub(crate) fn has_balances(&self, token_id: ContractTokenId, now: Timestamp) -> bool {
        self.tokens.get(&token_id).map_or(false, |token| {
            token
                .balances
                .iter()
                .any(|(_, balance)| balance.has_balance(now))
        })
    }

    /// Mints a new token balance.
    /// - If the token does not exist, an error is returned.
    /// - If the token balance already exists, the old balance is returned.
    pub(crate) fn mint(
        &mut self,
        token_id: ContractTokenId,
        account: AccountAddress,
        amount: ContractTokenAmount,
        expiry: Timestamp,
    ) -> ContractResult<Option<TokenBalanceState>> {
        match self.tokens.get_mut(&token_id) {
            Some(mut token) => Ok(token
                .balances
                .insert(account, TokenBalanceState { amount, expiry })),
            None => bail!(ContractError::InvalidTokenId),
        }
    }

    /// Get Account balance for a token.
    /// - If the token does not exist, InvalidTokenId is thrown.
    /// - If the account does not have a balance, 0 balance is returned.
    /// - If the balance has expired, 0 balance is returned.
    pub(crate) fn get_account_balance(
        &self,
        token_id: ContractTokenId,
        account: AccountAddress,
        now: Timestamp,
    ) -> ContractResult<ContractTokenAmount> {
        self.tokens
            .get(&token_id)
            .map_or(Err(ContractError::InvalidTokenId), |token| {
                Ok(token.get_account_balance(account, now))
            })
    }

    /// Get the Account Balance Expiry for a token.
    /// - If the token does not exist, InvalidTokenId is thrown.
    /// - If the account does not have a balance, None is returned.
    pub(crate) fn get_account_balance_expiry(
        &self,
        token_id: ContractTokenId,
        account: AccountAddress,
    ) -> ContractResult<Option<Timestamp>> {
        self.tokens
            .get(&token_id)
            .map_or(Err(ContractError::InvalidTokenId), |token| {
                Ok(token.get_account_balance_expiry(account))
            })
    }

    /// Gets the token metadata of the given token.
    /// - If the token does not exist, InvalidTokenId is thrown.
    pub(crate) fn get_token_metadata(
        &self,
        token_id: &ContractTokenId,
    ) -> ContractResult<MetadataUrl> {
        self.tokens
            .get(token_id)
            .map_or(Err(ContractError::InvalidTokenId), |token| {
                Ok(token.metadata.clone())
            })
    }
}
