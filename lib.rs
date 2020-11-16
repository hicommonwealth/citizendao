#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract(dynamic_storage_allocator = true)]
mod citizendao {
    const MAX_CANDIDATES : u32 = 10;
    const VOTES_TO_SETTLE : usize = 9;
    const VOTES_TO_ACCEPT : u32 = 7;

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
        pub fn num_candidates(&self) -> u32 {
            self.candidates.len()
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
            if self.candidates.len() >= MAX_CANDIDATES {
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

            // check for existing votes
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
                if *vote == true { yes_votes += 1; } // get preexisting number of yes votes
            }

            // create vote
            votes.push((caller, value));
            let total_votes = votes.len();
            self.env().emit_event(Voted { by: caller, on, value });
            if value == true { yes_votes += 1; }

            ink_env::debug_println(&format!( "voted! ({:?}/{:?})", yes_votes, total_votes ));

            // settle vote, if necessary
            if total_votes == self.members.len() || max_index == VOTES_TO_SETTLE - 1 {
                ink_env::debug_println(&format!( "settling candidacy! ({:?}/{:?})", yes_votes, total_votes ));

                match self.search_candidates(on) {
                    Some(index) => self.candidates.swap_remove_drop(index as u32).unwrap(),
                    None => { return Err(Error::UnexpectedError); }
                }
                self.votes.take(&on);
                if (total_votes == self.members.len() && total_votes == yes_votes) || yes_votes >= VOTES_TO_ACCEPT {
                    let member_record = (Self::env().block_timestamp(),);
                    self.members.insert(on, member_record);
                    self.env().emit_event(Decided { candidate: on, outcome: true });
                    ink_env::debug_println("member accepted");
                } else {
                    self.env().emit_event(Decided { candidate: on, outcome: false });
                    ink_env::debug_println("member rejected");
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
        fn candidate_selection_works() {
            let accounts = default_accounts();
            set_next_caller(accounts.alice);
            let mut contract = CitizenDAO::new();

            // alice votes yes on bob, bob is immediately in the DAO
            set_next_caller(accounts.bob);
            assert_eq!(contract.submit_candidacy(), Ok(()));
            assert_eq!(contract.num_candidates(), 1);
            set_next_caller(accounts.alice);
            assert_eq!(contract.vote_candidacy(accounts.bob, true), Ok(()));
            assert_eq!(contract.is_member(accounts.alice), true);
            assert_eq!(contract.is_member(accounts.bob), true);
            assert_eq!(contract.num_candidates(), 0);

            // alice and bob vote yes on charlie
            set_next_caller(accounts.charlie);
            assert_eq!(contract.submit_candidacy(), Ok(()));
            assert_eq!(contract.num_candidates(), 1);
            set_next_caller(accounts.alice);
            assert_eq!(contract.vote_candidacy(accounts.charlie, true), Ok(()));
            set_next_caller(accounts.bob);
            assert_eq!(contract.vote_candidacy(accounts.charlie, true), Ok(()));
            assert_eq!(contract.is_member(accounts.charlie), true);
            assert_eq!(contract.num_candidates(), 0);

            // django, eve, and frank are rejected from the DAO
            set_next_caller(accounts.django);
            assert_eq!(contract.submit_candidacy(), Ok(()));
            set_next_caller(accounts.eve);
            assert_eq!(contract.submit_candidacy(), Ok(()));
            set_next_caller(accounts.frank);
            assert_eq!(contract.submit_candidacy(), Ok(()));
            assert_eq!(contract.num_candidates(), 3);

            set_next_caller(accounts.alice);
            assert_eq!(contract.vote_candidacy(accounts.django, true), Ok(()));
            assert_eq!(contract.vote_candidacy(accounts.eve, false), Ok(()));
            assert_eq!(contract.vote_candidacy(accounts.frank, false), Ok(()));
            set_next_caller(accounts.bob);
            assert_eq!(contract.vote_candidacy(accounts.django, true), Ok(()));
            assert_eq!(contract.vote_candidacy(accounts.eve, false), Ok(()));
            assert_eq!(contract.vote_candidacy(accounts.frank, true), Ok(()));
            set_next_caller(accounts.charlie);
            assert_eq!(contract.vote_candidacy(accounts.django, false), Ok(()));
            assert_eq!(contract.vote_candidacy(accounts.eve, false), Ok(()));
            assert_eq!(contract.vote_candidacy(accounts.frank, true), Ok(()));
            assert_eq!(contract.num_candidates(), 0);
            assert_eq!(contract.is_member(accounts.django), false);
            assert_eq!(contract.is_member(accounts.eve), false);
            assert_eq!(contract.is_member(accounts.frank), false);
        }

        #[ink::test]
        fn ensure_no_duplicate_voting() {
        }
        #[ink::test]
        fn ensure_no_duplicate_candidates() {
        }
        #[ink::test]
        fn can_retrieve_candidate_set() {
        }
        #[ink::test]
        fn candidacy_queue_fills() {
        }
    }
}
