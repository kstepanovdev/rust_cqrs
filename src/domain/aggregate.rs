use async_trait::async_trait;
use cqrs_es::{Aggregate, AggregateError};
use serde::{Serialize, Deserialize};

use crate::domain::commands::BankAccountCommand;
use crate::domain::events::{BankAccountError, BankAccountEvent};
use crate::services::BankAccountServices;

#[derive(Serialize, Default, Deserialize)]
pub struct BankAccount {
    opened: bool,
    balance: f64
}

#[async_trait]
impl Aggregate for BankAccount {
    type Command = BankAccountCommand;
    type Event = BankAccountEvent;
    type Error = BankAccountError;
    type Services = BankAccountServices;

    fn aggregate_type() -> String {
        "Account".to_string()        
    }
    
    async fn handle(&self, command: Self::Command, services: &Self::Services) -> Result<Vec<Self::Event>, Self::Error> {
        match command {
            BankAccountCommand::DepositMoney { amount } => {
                let balance = self.balance + amount;
                Ok(vec![BankAccountEvent::CustomerDepositedMoney {
                    amount,
                    balance,
                }])
            }
            BankAccountCommand::WithdrawMoney { amount } => {
                let balance = self.balance - amount;
                if balance < 0_f64 {
                    return Err(BankAccountError::from("funds not available"));
                }
                Ok(vec![BankAccountEvent::CustomerWithdrewCash { amount, balance }])
            }
            _ => Ok(vec![])
        }
    }
          
    fn apply(&mut self, event: Self::Event) {
        match event {
           BankAccountEvent::AccountOpened { account_id } => {
               self.opened = true;
           }
           BankAccountEvent::CustomerDepositedMoney { amount, balance } => {
               self.balance = balance;
           }
           BankAccountEvent::CustomerWithdrewCash { amount, balance } => {
               self.balance = balance;
           }
           BankAccountEvent::CustomerWroteCheck { check_number, amount, balance } => {
               self.balance = balance;
           }
        }
    }
}

#[cfg(test)]
mod aggregate_tests {
    use super::*;
    use crate::domain::commands::BankAccountCommand::*;
    use cqrs_es::test::TestFramework;

    type AccountTestFramework = TestFramework<BankAccount>;
    
    #[test]
    fn test_deposit_money() {
        let expected = BankAccountEvent::CustomerDepositedMoney { amount: 200.0, balance: 200.0 };

        AccountTestFramework::with(BankAccountServices)
            .given_no_previous_events()
            .when(DepositMoney{ amount: 200.0 })
            .then_expect_events(vec![expected]);
    }
    
    #[test]
    fn test_deposit_money_with_balance() {
        let previous = BankAccountEvent::CustomerDepositedMoney { amount: 200.0, balance: 200.0 };
        let expected = BankAccountEvent::CustomerDepositedMoney { amount: 200.0, balance: 400.0 };

        AccountTestFramework::with(BankAccountServices)
            .given(vec![previous])
            .when(DepositMoney{ amount: 200.0 })
            .then_expect_events(vec![expected]);
    }
    #[test]
    fn test_withdraw_money() {
        let previous = BankAccountEvent::CustomerDepositedMoney { amount: 200.0, balance: 200.0 };
        let expected = BankAccountEvent::CustomerWithdrewCash { amount: 100.0, balance: 100.0 };
        
        AccountTestFramework::with(BankAccountServices)
            .given(vec![previous]) 
            .when(WithdrawMoney { amount: 100.0 })
            .then_expect_events(vec![expected])
    }
    
    #[test]
    fn test_withdraw_money_funds_not_available() {
        AccountTestFramework::with(BankAccountServices)
            .given_no_previous_events() 
            .when(WithdrawMoney { amount: 200.0 } )
            .then_expect_error_message("funds not available");
    }
}