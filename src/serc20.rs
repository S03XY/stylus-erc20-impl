use alloc::{string::String, vec::Vec};
use alloy_primitives::{Address, U256};
use core::marker::PhantomData;
use stylus_sdk::stylus_proc::{external, sol_storage};
use stylus_sdk::{evm, msg};

use stylus_sdk::alloy_sol_types::{sol, SolError};

pub trait SERC20Details {
    const NAME: &'static str;
    const SYMBOL: &'static str;
    const DECIMALS: u8;
}

sol_storage! {
    pub struct SERC20<T> {
        mapping (address => uint256) balances;
        mapping (address => mapping(address=> uint256 )) allowanace;
        uint256 total_supply;
        PhantomData<T> phantom;
    }

}

sol! {
    event Transfer(address indexed from,address indexed to,uint256 amount);
    event Approval(address indexed owner,address indexed spender,uint256 value);

    error InsufficientBalance(address from, uint256 have,uint256 want);
    error InsufficientAllowance(address owner,address spender,uint256 have,uint256 want);
}

pub enum SERC20Error {
    InsufficientBalance(InsufficientBalance),
    InsufficientAllowance(InsufficientAllowance),
}

impl From<SERC20Error> for Vec<u8> {
    fn from(err: SERC20Error) -> Vec<u8> {
        match err {
            SERC20Error::InsufficientBalance(e) => e.encode(),
            SERC20Error::InsufficientAllowance(e) => e.encode(),
        }
    }
}

// internal functions to the contract
impl<T: SERC20Details> SERC20<T> {
    
    pub fn transfer_impl(
        &mut self,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<(), SERC20Error> {
        let mut sender_balance = self.balances.setter(from);
        let old_sender_balance = sender_balance.get();

        if old_sender_balance < value {
            return Err(SERC20Error::InsufficientBalance(InsufficientBalance {
                from,
                have: old_sender_balance,
                want: value,
            }));
        }

        sender_balance.set(old_sender_balance - value);
        let mut to_balance = self.balances.setter(to);
        let new_balance = to_balance.get();
        to_balance.set(new_balance + value);
        evm::log(Transfer {
            from,
            to,
            amount: value,
        });

        Ok(())
    }

    pub fn mint(&mut self, address: Address, value: U256) {
        let mut balance = self.balances.setter(address);
        let new_balance = balance.get() + value;
        balance.set(new_balance);
        self.total_supply.set(self.total_supply.get() + value);
        evm::log(Transfer {
            from: Address::ZERO,
            to: address,
            amount: value,
        });
    }

    pub fn burn(&mut self, address: Address, value: U256) -> Result<(), SERC20Error> {
        let mut balance = self.balances.setter(address);
        let old_balance = balance.get();
        if old_balance < value {
            return Err(SERC20Error::InsufficientBalance(InsufficientBalance {
                from: address,
                have: old_balance,
                want: value,
            }));
        }
        balance.set(old_balance - value);
        self.total_supply.set(self.total_supply.get() - value);
        evm::log(Transfer {
            from: address,
            to: Address::ZERO,
            amount: value,
        });
        Ok(())
    }
}

// external function to the contract
#[external]
impl<T: SERC20Details> SERC20<T> {
    pub fn name() -> Result<String, SERC20Error> {
        Ok(T::NAME.into())
    }

    pub fn symbol() -> Result<String, SERC20Error> {
        Ok(T::SYMBOL.into())
    }

    pub fn decimals() -> Result<u8, SERC20Error> {
        Ok(T::DECIMALS)
    }

    pub fn balance_of(&self, address: Address) -> Result<U256, SERC20Error> {
        Ok(self.balances.get(address))
    }

    pub fn allowance(&self, owner: Address, spender: Address) -> Result<U256, SERC20Error> {
        Ok(self.allowanace.getter(owner).get(spender))
    }

    pub fn transfer(&mut self, to: Address, value: U256) -> Result<bool, SERC20Error> {
        self.transfer_impl(msg::sender(), to, value)?;
        Ok(true)
    }

    pub fn approve(&mut self, spender: Address, value: U256) -> Result<bool, SERC20Error> {
        self.allowanace.setter(msg::sender()).insert(spender, value);
        evm::log(Approval {
            owner: msg::sender(),
            spender,
            value,
        });
        Ok(true)
    }

    pub fn transfer_from(
        &mut self,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<bool, SERC20Error> {
        let mut sender_allowance = self.allowanace.setter(from);
        let mut allowance = sender_allowance.setter(msg::sender());
        let old_allowance = allowance.get();

        if old_allowance < value {
            return Err(SERC20Error::InsufficientAllowance(InsufficientAllowance {
                owner: from,
                spender: msg::sender(),
                want: value,
                have: old_allowance,
            }));
        }

        allowance.set(old_allowance - value);
        self.transfer_impl(from, to, value)?;

        Ok(true)
    }
}
