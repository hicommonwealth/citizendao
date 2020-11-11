#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract(dynamic_storage_allocator = true)]
mod citizendao {
    const MAX_CANDIDATES : u32 = 10;
    const VOTES_TO_SETTLE : usize = 9;
    const VOTES_TO_ACCEPT : usize = 7;

    #[cfg(not(feature = "ink-as-dependency"))]
    #[ink(storage)]
    // TODO: avoid duplicate candidates and votes data structures
    // TODO: identify and note of all places where we perform unrestricted iteration
    pub struct CitizenDAO {
        candidates: ink_storage::collections::Vec<AccountId>,
        votes: ink_storage::collections::HashMap<
                AccountId, ink_storage::Box<ink_storage::collections::Vec<(AccountId, bool)>>
                >,
        members: ink_storage::collections::HashMap<AccountId, (u64,)>,
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
    pub enum Error {
        CandidateQueueFull,
        NotAMember,
        UnexpectedError, // undefined behavior
    }
    pub type Result<T> = core::result::Result<T, Error>;

    #[ink(event)]
    pub struct Voted {
        #[ink(topic)]
        by: AccountId,
        #[ink(topic)]
        on: AccountId,
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
            let member_record = (Self::env().block_timestamp(),);
            members.insert(caller, member_record);

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
            if self.members.get(&account).is_some() {
                return true
            } else {
                return false
            }
        }

        #[ink(message)]
        pub fn is_candidate_or_member(&self, account: AccountId) -> bool {
            if self.is_member(account) { return true }

            if self.search_candidates(account).is_some() {
                return true
            } else {
                return false
            }
        }

        #[ink(message)]
        pub fn submit_candidacy(&mut self) -> Result<()> {
            if self.candidates.len() <= MAX_CANDIDATES {
                return Err(Error::CandidateQueueFull);
            }

            let caller = self.env().caller();
            let votes = ink_storage::collections::Vec::new();
            self.candidates.push(caller);
            self.votes.insert(caller, ink_storage::Box::new(votes));
            Ok(())
        }

        #[ink(message)]
        pub fn vote_candidacy(&mut self, on: AccountId, value: bool) -> Result<()> {
            let caller = self.env().caller();
            if !self.is_member(caller) {
                return Err(Error::NotAMember);
            }

            let votes = self.votes.get_mut(&on).unwrap();
            let mut yes_votes = 0;
            let mut max_index = usize::MAX;
            for (index, (voter, vote)) in votes.iter().enumerate() {
                if *voter == caller {
                    // overwrite vote
                    if let Err(_) = votes.set(index as u32, (caller, value)) {
                        return Err(Error::UnexpectedError);
                    }
                    self.env().emit_event(Voted { by: caller, on, value });
                    return Ok(());
                }
                max_index = index;
                if *vote == true { yes_votes += 1; }
            }

            // create vote
            votes.push((caller, value));
            self.env().emit_event(Voted { by: caller, on, value });

            // settle vote, if necessary
            if max_index + 1 == VOTES_TO_SETTLE {
                // self.candidates.remove(on);
                self.votes.take(&on);
                if yes_votes >= VOTES_TO_ACCEPT {
                    let member_record = (Self::env().block_timestamp(),);
                    self.members.insert(on, member_record);
                    self.env().emit_event(Decided { candidate: on, outcome: true });
                } else {
                    self.env().emit_event(Decided { candidate: on, outcome: false });
                }
            }
            Ok(())
        }

        fn search_candidates(&self, account: AccountId) -> Option<usize> {
            for (index, candidate) in self.candidates.iter().enumerate() {
                if *candidate == account { return Some(index); }
            }
            None
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

        #[ink::test]
        fn contract_works() {
            // TODO!

            // ensure_no_duplicate_voting
            // ensure_no_duplicate_candidates
            // ensure_votes_settle_and_candidate_accepted
            // ensure_votes_settle_and_candidate_rejected
            // can_retrieve_candidate_set
            // candidacy_queue_fills
        }
    }
}
