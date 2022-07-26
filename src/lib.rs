use solana_program::program::{invoke_signed, invoke};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    msg,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction,
    sysvar::{clock::Clock, Sysvar, rent::Rent},
    self,
    program_pack::Pack,
};
use solana_program::borsh::try_from_slice_unchecked;
use borsh::{BorshDeserialize, BorshSerialize,BorshSchema};
use spl_token;
use spl_associated_token_account;
use spl_token_metadata;


// Declare and export the program's entrypoint
entrypoint!(process_instruction);

#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
enum StakeInstruction{
    GenerateVault,
    Submit,
    Claim,
}

#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
struct StakeData{
    sender: Pubkey,
    nft: Pubkey,
    reciever: Pubkey,
    reciever_nft: Pubkey,
    active: bool,
    claim: bool,
}


// Program entrypoint's implementation
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let instruction: StakeInstruction = try_from_slice_unchecked(instruction_data).unwrap();
    let vault_word = "vault";

    let admin = "HRqXXua5SSsr1C7pBWhtLxjD9HcreNd4ZTKJD7em7mtP".parse::<Pubkey>().unwrap();

    match instruction{
        
        StakeInstruction::Claim=>{
            let payer = next_account_info(accounts_iter)?;
            let system_program = next_account_info(accounts_iter)?;
            let nft_info = next_account_info(accounts_iter)?;
            let nft_one_info = next_account_info(accounts_iter)?;
            let token_info = next_account_info(accounts_iter)?;
            let rent_info = next_account_info(accounts_iter)?;
            let assoc_acccount_info = next_account_info(accounts_iter)?;
            let stake_info = next_account_info(accounts_iter)?;
            let stake_one_info = next_account_info(accounts_iter)?;
            let vault_info = next_account_info(accounts_iter)?;
            let payer_nft_holder_info = next_account_info(accounts_iter)?;
            let vault_nft_holder_info = next_account_info(accounts_iter)?;
           

            let ( stake_address, _stake_bump ) = Pubkey::find_program_address(&[&nft_info.key.to_bytes()], &program_id);
            let ( stake_address_one, _stake_bump ) = Pubkey::find_program_address(&[&nft_one_info.key.to_bytes()], &program_id);
            let ( vault_address, vault_bump ) = Pubkey::find_program_address(&[&vault_word.as_bytes()], &program_id);
            let payer_nft_holder = spl_associated_token_account::get_associated_token_address(payer.key, nft_info.key);
            let vault_nft_holder = spl_associated_token_account::get_associated_token_address(vault_info.key, nft_info.key);
            

            // let holder_data = spl_token::state::Account::unpack_from_slice(&payer_nft_holder_info.data.borrow()[..]).unwrap();
            
            // if holder_data.amount!=1{
            //     //no ticket in wallet
            //     return Err(ProgramError::Custom(0x102));
            // }
            if *token_info.key!=spl_token::id(){
                //wrong token_info
                return Err(ProgramError::Custom(0x345));
            }

            if stake_address!=*stake_info.key{
                //wrong stake_info
                return Err(ProgramError::Custom(0x60));
            }
            if stake_address_one!=*stake_one_info.key{
                //wrong stake_info
                return Err(ProgramError::Custom(0x60));
            }

            if vault_address!=*vault_info.key{
                //wrong stake_info
                return Err(ProgramError::Custom(0x61));
            }

            if payer_nft_holder!=*payer_nft_holder_info.key{
                //wrong payer_nft_holder_info
                return Err(ProgramError::Custom(0x64));
            }

            if vault_nft_holder!=*vault_nft_holder_info.key{
                //wrong vault_nft_holder_info
                return Err(ProgramError::Custom(0x65));
            }

            let mut stake_data = if let Ok(data) = StakeData::try_from_slice(&stake_info.data.borrow()){
                data
            } else {
                // can't deserialize stake data
                return Err(ProgramError::Custom(0x913));
            };
            let mut stake_data_one = if let Ok(data) = StakeData::try_from_slice(&stake_one_info.data.borrow()){
                data
            } else {
                // can't deserialize stake data
                return Err(ProgramError::Custom(0x913));
            };

            if !stake_data.active{
                //both NFTs are not in vault
                return Err(ProgramError::Custom(0x107));
            }
            if !stake_data_one.active{
                //both NFTs are not in vault
                return Err(ProgramError::Custom(0x108));
            }

            if stake_data.reciever!=*payer.key{
                //unauthorized access
                return Err(ProgramError::Custom(0x109));
            }
            if stake_data.claim || stake_data_one.claim
            {
                stake_data.active=false;
                stake_data.serialize(&mut &mut stake_info.data.borrow_mut()[..])?;
                stake_data_one.active=false;
                stake_data.serialize(&mut &mut stake_one_info.data.borrow_mut()[..])?;
            }
                if payer_nft_holder_info.owner != token_info.key{
                    invoke(
                        &spl_associated_token_account::create_associated_token_account(
                            payer.key,
                            payer.key,
                            nft_info.key,
                        ),
                        &[
                            payer.clone(), 
                            payer_nft_holder_info.clone(), 
                            payer.clone(),
                            nft_info.clone(),
                            system_program.clone(),
                            token_info.clone(),
                            rent_info.clone(),
                            assoc_acccount_info.clone(),
                        ],
                        
                    )?;
                }
    
                invoke_signed(
                    &spl_token::instruction::transfer(
                        token_info.key,
                        vault_nft_holder_info.key,
                        payer_nft_holder_info.key,
                        vault_info.key,
                        &[],
                        1,
                    )?,
                    &[
                        vault_nft_holder_info.clone(),
                        payer_nft_holder_info.clone(),
                        vault_info.clone(), 
                        token_info.clone()
                    ],
                    &[&[&vault_word.as_bytes(), &[vault_bump]]],
                )?;
    
                invoke_signed(
                    &spl_token::instruction::close_account(
                        token_info.key,
                        vault_nft_holder_info.key,
                        payer.key,
                        vault_info.key,
                        &[],
                    )?,
                    &[
                        vault_nft_holder_info.clone(),
                        payer.clone(),
                        vault_info.clone(), 
                        token_info.clone()
                    ],
                    &[&[&vault_word.as_bytes(), &[vault_bump]]],
                )?;
                stake_data.claim=true;
                stake_data.serialize(&mut &mut stake_info.data.borrow_mut()[..])?;
            
           
        },
        
        StakeInstruction::Submit=>{
            let payer = next_account_info(accounts_iter)?;
            let reciever = next_account_info(accounts_iter)?;
            let mint = next_account_info(accounts_iter)?;
            let mint_recieve = next_account_info(accounts_iter)?;
            let metadata_account_info = next_account_info(accounts_iter)?;
            
            let vault_info = next_account_info(accounts_iter)?;
            let source = next_account_info(accounts_iter)?;
            let destination = next_account_info(accounts_iter)?;

            let token_program = next_account_info(accounts_iter)?;
            let sys_info = next_account_info(accounts_iter)?;
            let rent_info = next_account_info(accounts_iter)?;
            let token_assoc = next_account_info(accounts_iter)?;
            
            let stake_data_info = next_account_info(accounts_iter)?;

           
            msg!("mint_recieve {:?}",mint_recieve);
            //let holder_data = spl_token::state::Account::unpack_from_slice(&source.data.borrow()[..]).unwrap();
            // if holder_data.amount!=1{
            //     //no ticket in wallet
            //     return Err(ProgramError::Custom(0x102));
            // }
            if *token_program.key!=spl_token::id(){
                //wrong token_info
                return Err(ProgramError::Custom(0x345));
            }

            let rent = &Rent::from_account_info(rent_info)?;
            let ( stake_data, stake_data_bump ) = Pubkey::find_program_address(&[&mint.key.to_bytes()], &program_id);

            if !payer.is_signer{
                //unauthorized access
                return Err(ProgramError::Custom(0x11));
            }

            if stake_data!=*stake_data_info.key{
                //msg!("invalid stake_data account!");
                return Err(ProgramError::Custom(0x10));
            }

            let size: u64 = 32+32+32+32+1+1;
            if stake_data_info.owner != program_id{
                let required_lamports = rent
                .minimum_balance(size as usize)
                .max(1)
                .saturating_sub(stake_data_info.lamports());
                invoke(
                    &system_instruction::transfer(payer.key, &stake_data, required_lamports),
                    &[
                        payer.clone(),
                        stake_data_info.clone(),
                        sys_info.clone(),
                    ],
                )?;
                invoke_signed(
                    &system_instruction::allocate(&stake_data, size),
                    &[
                        stake_data_info.clone(),
                        sys_info.clone(),
                    ],
                    &[&[&mint.key.to_bytes(), &[stake_data_bump]]],
                )?;

                invoke_signed(
                    &system_instruction::assign(&stake_data, program_id),
                    &[
                        stake_data_info.clone(),
                        sys_info.clone(),
                    ],
                    &[&[&mint.key.to_bytes(), &[stake_data_bump]]],
                )?;
            }

            let stake_struct = StakeData{
                sender: *payer.key,
                nft: *mint.key,
                reciever: *reciever.key,
                reciever_nft: *mint_recieve.key,
                active: true,
                claim:false,
            };
            stake_struct.serialize(&mut &mut stake_data_info.data.borrow_mut()[..])?;

            if &Pubkey::find_program_address(&["metadata".as_bytes(), &spl_token_metadata::ID.to_bytes(), &mint.key.to_bytes()], &spl_token_metadata::ID).0 != metadata_account_info.key {
                //msg!("invalid metadata account!");
                return Err(ProgramError::Custom(0x03));
            }

            let ( vault, _vault_bump ) = Pubkey::find_program_address(&[&vault_word.as_bytes()], &program_id);
            if vault != *vault_info.key{
                //msg!("Wrong vault");
                return Err(ProgramError::Custom(0x07));
            }

            if &spl_associated_token_account::get_associated_token_address(payer.key, mint.key) != source.key {
                // msg!("Wrong source");
                return Err(ProgramError::Custom(0x08));
            }

            if &spl_associated_token_account::get_associated_token_address(&vault, mint.key) != destination.key{
                //msg!("Wrong destination");
                return Err(ProgramError::Custom(0x09));
            }

            if destination.owner != token_program.key{
                invoke(
                    &spl_associated_token_account::create_associated_token_account(
                        payer.key,
                        vault_info.key,
                        mint.key,
                    ),
                    &[
                        payer.clone(), 
                        destination.clone(), 
                        vault_info.clone(),
                        mint.clone(),
                        sys_info.clone(),
                        token_program.clone(),
                        rent_info.clone(),
                        token_assoc.clone(),
                    ],
                )?;
            }
            invoke(
                &spl_token::instruction::transfer(
                    token_program.key,
                    source.key,
                    destination.key,
                    payer.key,
                    &[],
                    1,
                )?,
                &[
                    source.clone(),
                    destination.clone(),
                    payer.clone(), 
                    token_program.clone()
                ],
            )?;

        },

        StakeInstruction::GenerateVault=>{
            let payer = next_account_info(accounts_iter)?;
            let system_program = next_account_info(accounts_iter)?;
            let pda = next_account_info(accounts_iter)?;
            let rent_info = next_account_info(accounts_iter)?;

            let rent = &Rent::from_account_info(rent_info)?;

            let (vault_pda, vault_bump_seed) =
                Pubkey::find_program_address(&[vault_word.as_bytes()], &program_id);
            
            if pda.key!=&vault_pda{
                //msg!("Wrong account generated by client");
                return Err(ProgramError::Custom(0x00));
            }

            if *payer.key!=admin||!payer.is_signer{
                //unauthorized access
                return Err(ProgramError::Custom(0x02));
            }

            
        }
    };
        
    Ok(())
}


