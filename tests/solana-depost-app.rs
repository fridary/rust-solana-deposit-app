import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SolanaDepositApp } from "../target/types/solana_deposit_app";
import { PublicKey } from '@solana/web3.js';
import { expect } from 'chai';

describe("solana-deposit-app", () => {
  // Настройка провайдера
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.SolanaDepositApp as Program<SolanaDepositApp>;
  const user = provider.wallet.publicKey;
  
  let vaultKeypair: anchor.web3.Keypair;
  let vaultAuthority: PublicKey;
  let vaultBump: number;

  before(async () => {
    vaultKeypair = anchor.web3.Keypair.generate();
    
    // Найти PDA для vault authority
    const [authority, bump] = await PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), vaultKeypair.publicKey.toBuffer()],
      program.programId
    );
    
    vaultAuthority = authority;
    vaultBump = bump;
  });

  it("Инициализирует хранилище", async () => {
    await program.methods
      .initialize()
      .accounts({
        vault: vaultKeypair.publicKey,
        user: user,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([vaultKeypair])
      .rpc();

    // Проверка, что хранилище создано
    const vault = await program.account.vault.fetch(vaultKeypair.publicKey);
    expect(vault.owner.toString()).to.equal(user.toString());
    expect(vault.balance.toNumber()).to.equal(0);
  });

  it("Делает депозит SOL", async () => {
    const depositAmount = new anchor.BN(1000000000); // 1 SOL в лампортах
    
    await program.methods
      .deposit(depositAmount)
      .accounts({
        vault: vaultKeypair.publicKey,
        vaultAuthority: vaultAuthority,
        user: user,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    // Проверка баланса
    const vault = await program.account.vault.fetch(vaultKeypair.publicKey);
    expect(vault.balance.toString()).to.equal(depositAmount.toString());
  });

  it("Проверяет баланс", async () => {
    await program.methods
      .checkBalance()
      .accounts({
        vault: vaultKeypair.publicKey,
        user: user,
      })
      .rpc();

    // Получаем баланс напрямую
    const vault = await program.account.vault.fetch(vaultKeypair.publicKey);
    console.log(`Текущий баланс: ${vault.balance.toString()} лампортов`);
  });

  it("Выводит часть средств", async () => {
    const withdrawAmount = new anchor.BN(500000000); // 0.5 SOL
    
    await program.methods
      .withdraw(withdrawAmount)
      .accounts({
        vault: vaultKeypair.publicKey,
        vaultAuthority: vaultAuthority,
        user: user,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    // Проверка баланса после вывода
    const vault = await program.account.vault.fetch(vaultKeypair.publicKey);
    expect(vault.balance.toString()).to.equal("500000000"); // 0.5 SOL осталось
  });
});