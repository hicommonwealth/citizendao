#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

// let VOTES_TO_DECIDE = 9;
// let VOTES_TO_DECIDE_YES = 6;

#[ink::contract]
mod citizendao {
    #[cfg(not(feature = "ink-as-dependency"))]
    #[ink(storage)]
    pub struct CitizenDAO {
        candidates: ink_storage::collections::Vec<AccountId>,
        votes: ink_storage::collections::HashMap<AccountId, (AccountId, bool)>,
        members: ink_storage::collections::HashMap<AccountId, bool>,
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
    pub enum Error {
        UnimplementedError,
    }
    pub type Result<T> = core::result::Result<T, Error>;

    #[ink(event)]
    pub struct Voted {
        #[ink(topic)]
        by: Option<AccountId>,
        #[ink(topic)]
        on: Option<AccountId>,
        #[ink(topic)]
        value: bool
    }

    #[ink(event)]
    pub struct Decided {
        #[ink(topic)]
        candidate: AccountId,
        #[ink(topic)]
        outcome: bool
    }

    impl CitizenDAO {
        #[ink(constructor)]
        pub fn new() -> Self {
            let caller = Self::env().caller();
            let candidates = ink_storage::collections::Vec::new();
            let votes = ink_storage::collections::HashMap::new();
            let mut members = ink_storage::collections::HashMap::new();
            members.insert(caller, true);

            Self::env().emit_event(
                Decided { candidate: caller, outcome: true }
            );

            Self {
                candidates,
                votes,
                members
            }
        }

        #[ink(message)]
        pub fn is_member(&self, account: AccountId) -> bool {
            // TODO
            true
        }

        #[ink(message)]
        pub fn is_candidate_or_member(&self, account: AccountId) -> bool {
            if self.is_member(account) { return true }
            // TODO
            false
        }

        #[ink(message)]
        pub fn submit_candidacy(&self, attestation_link: Vec<u32>) -> Result<()> {
            return Err(Error::UnimplementedError);
            // TODO
            Ok(())
        }

        #[ink(message)]
        pub fn vote_candidacy(&self, on: AccountId, value: bool) -> Result<()> {
            return Err(Error::UnimplementedError);
            // TODO
            Ok(())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink_lang as ink;

        const DEFAULT_CALLEE_HASH: [u8; 32] = [0x07; 32];
        const DEFAULT_ENDOWMENT: Balance = 1_000_000;
        const DEFAULT_GAS_LIMIT: Balance = 1_000_000;

        fn default_accounts(
        ) -> ink_env::test::DefaultAccounts<ink_env::DefaultEnvironment> {
            ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                .expect("off-chain environment should have been initialized already")
        }

        fn set_next_caller(caller: AccountId) {
            ink_env::test::push_execution_context::<ink_env::DefaultEnvironment>(
                caller,
                AccountId::from(DEFAULT_CALLEE_HASH),
                DEFAULT_ENDOWMENT,
                DEFAULT_GAS_LIMIT,
                ink_env::test::CallData::new(ink_env::call::Selector::new([0x00; 4])),
            )
        }

        #[ink::test]
        fn new_works() {
            let accounts = default_accounts();
            set_next_caller(accounts.alice);
            let contract = CitizenDAO::new();
            assert_eq!(contract.is_member(accounts.alice), true);
            assert_eq!(contract.is_member(accounts.bob), false);
        }
    }
}
