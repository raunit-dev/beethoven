use {
    beethoven::{try_from_withdraw_context, Withdraw, WithdrawContext, WithdrawData},
    pinocchio::{error::ProgramError, AccountView, ProgramResult},
};

/// Instruction data for Withdraw
///
/// Layout:
/// [0..8] - amount (u64, little-endian)
/// [8..] - protocol-specific data (parsed via WithdrawContext::try_from_withdraw_data)
pub struct WithdrawInstructionData<'a> {
    pub amount: u64,
    pub extra_data: &'a [u8],
}

impl<'a> TryFrom<&'a [u8]> for WithdrawInstructionData<'a> {
    type Error = ProgramError;

    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        if data.len() < 8 {
            return Err(ProgramError::InvalidInstructionData);
        }
        Ok(Self {
            amount: u64::from_le_bytes(data[0..8].try_into().unwrap()),
            extra_data: &data[8..],
        })
    }
}

pub struct WithdrawInstruction<'a> {
    pub accounts: WithdrawContext<'a>,
    pub data: WithdrawData,
    pub amount: u64,
}

impl<'a> TryFrom<(&'a [AccountView], &[u8])> for WithdrawInstruction<'a> {
    type Error = ProgramError;

    fn try_from((accounts, data): (&'a [AccountView], &[u8])) -> Result<Self, Self::Error> {
        let instruction_data = WithdrawInstructionData::try_from(data)?;
        let ctx = try_from_withdraw_context(accounts)?;
        let (withdraw_data, remaining_data) =
            ctx.try_from_withdraw_data(instruction_data.extra_data)?;
        if !remaining_data.is_empty() {
            return Err(ProgramError::InvalidInstructionData);
        }

        Ok(Self {
            accounts: ctx,
            data: withdraw_data,
            amount: instruction_data.amount,
        })
    }
}

impl<'a> WithdrawInstruction<'a> {
    pub fn process(&self) -> ProgramResult {
        WithdrawContext::withdraw(&self.accounts, self.amount, &self.data)
    }
}

pub fn process(accounts: &[AccountView], data: &[u8]) -> ProgramResult {
    WithdrawInstruction::try_from((accounts, data))?.process()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn withdraw_instruction_data_splits_amount_and_extra_data() {
        let amount = 42u64;
        let mut data = amount.to_le_bytes().to_vec();
        data.extend_from_slice(&[1, 1]);

        let parsed = WithdrawInstructionData::try_from(data.as_slice()).unwrap();
        assert_eq!(parsed.amount, amount);
        assert_eq!(parsed.extra_data, &[1, 1]);
    }

    #[test]
    fn withdraw_instruction_data_rejects_short_payload() {
        let err = match WithdrawInstructionData::try_from(&[1, 2, 3][..]) {
            Ok(_) => panic!("expected InvalidInstructionData"),
            Err(err) => err,
        };
        assert_eq!(err, ProgramError::InvalidInstructionData);
    }
}
