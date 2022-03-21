use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};

/**********************************************************************/
fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    if instruction_data.len() == 0 {
        return Err(ProgramError::InvalidInstructionData);
    }
    if instruction_data[0] == 0 {
        return create_campain(
            program_id, 
            accounts, 
            &instruction_data[1..instruction_data.len()],
        );
    }
    else if instruction_data[0] == 1 {
        return withdraw(
            program_id,
            accounts,
            &instruction_data[1..instruction_data.len()],
        );
    }
    else if instruction_data[0] == 2 {
        return donate(
            program_id,
            accounts,
            &instruction_data[1..instruction_data.len()],
        );
    }
    msg!("No entry_point found!!!");
    Err(ProgramError::InvalidInstructionData)
}

entrypoint!(process_instruction);

/**********************************************************************/
#[derive(BorshSerialize, BorshDeserialize, Debug)]

    struct CampainDetails {
        pub admin: Pubkey,
        pub name: String,
        pub description: String,
        pub image_link: String,
    /*
     * total amount donated to a campaign.
     */
        pub amount_donated: u64,
    }

/*----------------------------*/
fn create_campain(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {

    let accounts_iter = &mut accounts.iter();

    /*
     * Writing account or we can call it program account.
     * This is an account we will create in our front-end.
     * This account should br owned by the solana program.
     */
    let writing_account = next_account_info(accounts_iter)?;
    /*
     * Account of the person creating the campaign.
     */
    let creator_account = next_account_info(accounts_iter)?;
    /*
     * allow transactions just by the creator account by sign the transaction.
     */
    if !creator_account.is_signer {
        msg!("The creator_account is not a signer!!!");
        return Err(ProgramError::IncorrectProgramId);
    }
    if writing_account.owner != program_id {
        msg!("The program don't own the writing_account!!!");
        return Err(ProgramError::IncorrectProgramId);
    }

    /*
     * By deriving the trait "BorshDeserialize" in our "CampaignDetails" struct we have added a 
     * method "try_from_slice" which takes in the parameter array of [u8] and creates an object of 
     * "CampaignDetails" with it. It gives us an enum of type results. 
     * We will use the "expect" method on result enums to and pass in the string which we can see in case of error.

     */
    let mut input_data = CampainDetails::try_from_slice(&instruction_data)
    .expect("Instruction_data serialization faild!!");

    /*
     * for a campaign created the only admin should be the one who created it.
     */
    if input_data.admin != *creator_account.key {
        msg!("Wrong Instruction data!!!");
        return Err(ProgramError::InvalidInstructionData);
    }

    /*
     * get the minimum balance we need in our program account.
     */
    let rent_exemption = Rent::get()?.minimum_balance(writing_account.data_len());
    /*
     * make sure our program account (`writing_account`) has that much lamports(balance).
     */
    if **writing_account.lamports.borrow() < rent_exemption {
        msg!("The balance of writing_account is less than the rent_exemption ammount!!!");
        return Err(ProgramError::InsufficientFunds);
    }
    /*
     * initial amount donate to be zero.
     */
    input_data.amount_donated = 0;
    input_data.serialize(&mut &mut writing_account.try_borrow_mut_data()?[..])?;

    Ok(())
}

/**************************************/
#[derive(BorshSerialize, BorshDeserialize, Debug)]
struct WithdrawRequest {
    pub amount: u64,
}
/*----------------------------*/
fn withdraw(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {

    /*
     * create iterator and get "writing_account" (program owned account) and "admin_account".
     */
    let accounts_iter = &mut accounts.iter();
    let writing_account = next_account_info(accounts_iter)?;
    let admin_account = next_account_info(accounts_iter)?;

    /*
     * Admin account should be the signer in this trasaction.
     */
    if !admin_account.is_signer {
        msg!("Admin is not a signer!!!");
        return Err(ProgramError::IncorrectProgramId);
    }
    /*
     * check if the writing account is owned by program.
     */
    if writing_account.owner != program_id {
        msg!("Writing_account not owned bu the program!!!");
        return Err(ProgramError:: IncorrectProgramId);
    }
    /*
     * we will get the data of campaign from the writing_account
     * we stored this when we created the campaign with create_campaign function.
     */
    let campaign_data = CampainDetails::try_from_slice(*writing_account.data.borrow())
    .expect("Deserializing data faild!!!");

    if campaign_data.admin != *admin_account.key {
        msg!("Only the account admin can withdraw");
        return Err(ProgramError::InvalidAccountData);
    }
    /*
     * Here we make use of the struct we created.
     * We will get the amount of lamports admin wants to withdraw
     */
    let input_data = WithdrawRequest::try_from_slice(&instruction_data)
    .expect("Instruction serialization faild!!!");
    /*
     * We do not want the campaign to get deleted after a withdrawal. 
     * We want it to always have a minimum balance,
     * So we calculate the rent_exemption and consider it.
     */
    let rent_exemption = Rent::get()?.minimum_balance(writing_account.data_len());
    /*
     * check if we have enough funds
     */
    if **writing_account.lamports.borrow() - rent_exemption < input_data.amount {
        msg!("Not enough balance to keep the account alife!!!");
        return Err(ProgramError::InsufficientFunds);
    }
    /*
     * Transfer balance
     * decrease the balance of the program account, 
     * and increase the admin_account balance.
     */
    **writing_account.try_borrow_mut_lamports()? -= input_data.amount;
    **admin_account.try_borrow_mut_lamports()? += input_data.amount;

    Ok(())
}
/**************************************/
/*
 * We want to donate to a campaign, however we can't decrease the balance of an account not owned by our program in our program.
 * This means we can't just transfer the balance as we did in the withdraw function. 
 * Solana policies state: "An account not assigned to the program cannot have its balance decrease."
 * So for this, we will create a program-owned account in our front-end and then perform the SOL token transaction.
 */
fn donate(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {

    let accounts_iter = &mut accounts.iter();
    /*
     * Create 3 accounts here,
     * First is the program-owned account containing the data of campaign we want to donate to.
     * Second we have a donator_program_account which is also the program-owned account that only has the Lamport we would like to donate.
     * Third we have the account of the donator.
     */
    let writing_account = next_account_info(accounts_iter)?;
    let donator_program_account = next_account_info(accounts_iter)?;
    let donator = next_account_info(accounts_iter)?;

    if !donator.is_signer {
        msg!("The donator is not a signer!!!");
        return Err(ProgramError::IncorrectProgramId);
    }
    if writing_account.owner != program_id {
        msg!("writing_account not owned by the program!!!");
        return Err(ProgramError::IncorrectProgramId);
    }
    if donator_program_account.owner != program_id {
        msg!("donator_program_account not owned by the program!!!");
        return Err(ProgramError::IncorrectProgramId);
    }
    /*
     * get the campaign_data and we will increment the amount_donated.
     */
    let mut campaign_data = CampainDetails::try_from_slice(*writing_account.data.borrow())
    .expect("deserializing data faild!!!");
    campaign_data.amount_donated += **donator_program_account.lamports.borrow();
    /*
     * Then we do the actual transaction.
     * Note that the donator_program_account is owned by program so it can decrease its Lamports.
     */
    **writing_account.try_borrow_mut_lamports()? += **donator_program_account.lamports.borrow();
    **donator_program_account.try_borrow_mut_lamports()? = 0;

    /*
     * at the end of the program we will write the new updated "campaign_data" to the writing_account's data field
     * and return the result Ok(()).
     */
    campaign_data.serialize(&mut &mut writing_account.data.borrow_mut()[..])?;

    Ok(())
}
/**********************************************************************/
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}

/**********************************************************************/