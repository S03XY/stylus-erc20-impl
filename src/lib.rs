#![cfg_attr(not(feature = "export-abi"), no_main, no_std)]

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

extern crate alloc;

mod serc20;

use alloy_primitives::{Address, U256};
use serc20::{SERC20Details, SERC20Error, SERC20};
use stylus_sdk::stylus_proc::{entrypoint, external, sol_storage};

pub struct TestTokenDetails;

impl SERC20Details for TestTokenDetails {
    const NAME: &'static str = "Test Token";
    const SYMBOL: &'static str = "TTS";
    const DECIMALS: u8 = 18;
}

sol_storage! {
    #[entrypoint]
    pub struct TestToken {
        #[borrow]
        SERC20<TestTokenDetails> test_token;
    }

}

#[external]
#[inherit(SERC20<TestTokenDetails>)]
impl TestToken {
    pub fn mint(&mut self, address: Address, value: U256) -> Result<bool, SERC20Error> {
        self.test_token.mint(address, value);
        Ok(true)
    }
    pub fn burn(&mut self, address: Address, value: U256) -> Result<bool, SERC20Error> {
        self.test_token.burn(address, value)?;
        Ok(true)
    }
}
