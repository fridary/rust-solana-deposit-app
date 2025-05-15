use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod solana_deposit_app {
    use super::*;

    // Инициализация нового хранилища депозитов
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        vault.owner = ctx.accounts.user.key();
        vault.balance = 0;
        Ok(())
    }

    // Внесение депозита
    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        let user = &ctx.accounts.user;
        
        // Переводим SOL из кошелька пользователя в PDA
        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            anchor_lang::system_program::Transfer {
                from: user.to_account_info(),
                to: ctx.accounts.vault_authority.to_account_info(),
            },
        );
        
        anchor_lang::system_program::transfer(cpi_context, amount)?;
        
        // Обновляем баланс
        vault.balance = vault.balance.checked_add(amount).unwrap();
        
        Ok(())
    }

    // Снятие средств
    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        
        // Проверяем, достаточно ли средств
        if vault.balance < amount {
            return Err(ErrorCode::InsufficientFunds.into());
        }
        
        // Определяем seed для PDA
        let vault_bump = *ctx.bumps.get("vault_authority").unwrap();
        let seeds = &[
            b"vault".as_ref(),
            ctx.accounts.vault.to_account_info().key.as_ref(), 
            &[vault_bump]
        ];
        let signer = &[&seeds[..]];
        
        // Переводим SOL обратно пользователю
        let cpi_context = CpiContext::new_with_signer(
            ctx.accounts.system_program.to_account_info(),
            anchor_lang::system_program::Transfer {
                from: ctx.accounts.vault_authority.to_account_info(),
                to: ctx.accounts.user.to_account_info(),
            },
            signer,
        );
        
        anchor_lang::system_program::transfer(cpi_context, amount)?;
        
        // Обновляем баланс
        vault.balance = vault.balance.checked_sub(amount).unwrap();
        
        Ok(())
    }

    // Проверка баланса
    pub fn check_balance(ctx: Context<CheckBalance>) -> Result<()> {
        // Просто проверка баланса, ничего не меняем
        // Баланс можно получить из клиента
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = user, space = 8 + 32 + 8)]
    pub vault: Account<'info, Vault>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut, has_one = owner @ ErrorCode::Unauthorized)]
    pub vault: Account<'info, Vault>,
    /// CHECK: Это PDA счет для хранения SOL
    #[account(
        mut,
        seeds = [b"vault", vault.key().as_ref()],
        bump,
    )]
    pub vault_authority: UncheckedAccount<'info>,
    #[account(mut, constraint = user.key() == vault.owner @ ErrorCode::Unauthorized)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut, has_one = owner @ ErrorCode::Unauthorized)]
    pub vault: Account<'info, Vault>,
    /// CHECK: Это PDA счет для хранения SOL
    #[account(
        mut,
        seeds = [b"vault", vault.key().as_ref()],
        bump,
    )]
    pub vault_authority: UncheckedAccount<'info>,
    #[account(mut, constraint = user.key() == vault.owner @ ErrorCode::Unauthorized)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CheckBalance<'info> {
    pub vault: Account<'info, Vault>,
    pub user: Signer<'info>,
}

#[account]
pub struct Vault {
    pub owner: Pubkey,   // Владелец хранилища
    pub balance: u64,    // Текущий баланс в лампортах (1 SOL = 1_000_000_000 лампортов)
}

#[error_code]
pub enum ErrorCode {
    #[msg("Недостаточно средств для вывода")]
    InsufficientFunds,
    #[msg("Не авторизован")]
    Unauthorized,
}