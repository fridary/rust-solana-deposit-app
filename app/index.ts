import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Connection, PublicKey, Keypair, LAMPORTS_PER_SOL } from '@solana/web3.js';
import fs from 'fs';
import path from 'path';
import { SolanaDepositApp } from "../target/types/solana_deposit_app";

// Получение идентификатора из файла
function loadKeypair(file: string): Keypair {
  const keypairFile = fs.readFileSync(file, { encoding: 'utf8' });
  const keypairData = JSON.parse(keypairFile);
  return Keypair.fromSecretKey(new Uint8Array(keypairData));
}

async function main() {
  // Инициализация подключения к devnet
  const connection = new Connection("https://api.devnet.solana.com", "confirmed");
  
  // Загрузка кошелька
  const wallet = new anchor.Wallet(loadKeypair(path.resolve(process.env.HOME!, ".config/solana/id.json")));
  
  // Настройка провайдера
  const provider = new anchor.AnchorProvider(connection, wallet, {
    preflightCommitment: "confirmed",
  });
  
  anchor.setProvider(provider);
  
  // Загрузка программы
  const programId = new PublicKey("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");
  const program = new Program<SolanaDepositApp>(
    require("../target/idl/solana_deposit_app.json"),
    programId
  );
  
  // Создаем новый аккаунт хранилища
  const vaultKeypair = Keypair.generate();
  
  console.log("Ваш публичный ключ:", wallet.publicKey.toString());
  console.log("Создание хранилища с ключом:", vaultKeypair.publicKey.toString());

  try {
    // Инициализация хранилища
    await program.methods
      .initialize()
      .accounts({
        vault: vaultKeypair.publicKey,
        user: wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([vaultKeypair])
      .rpc();
    
    console.log("Хранилище успешно инициализировано");
    
    // Находим PDA для хранения SOL
    const [vaultAuthority, _] = await PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), vaultKeypair.publicKey.toBuffer()],
      program.programId
    );
    
    console.log("Authority PDA:", vaultAuthority.toString());
    
    // Внесение депозита
    const depositAmount = new anchor.BN(0.1 * LAMPORTS_PER_SOL); // 0.1 SOL
    
    await program.methods
      .deposit(depositAmount)
      .accounts({
        vault: vaultKeypair.publicKey,
        vaultAuthority: vaultAuthority,
        user: wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();
    
    console.log(`Депозит ${depositAmount.toString()} лампортов (${depositAmount.toNumber() / LAMPORTS_PER_SOL} SOL) успешно внесен`);
    
    // Проверка баланса
    const vaultAccount = await program.account.vault.fetch(vaultKeypair.publicKey);
    console.log(`Текущий баланс: ${vaultAccount.balance.toString()} лампортов (${vaultAccount.balance.toNumber() / LAMPORTS_PER_SOL} SOL)`);
    
    // Вывод части средств
    const withdrawAmount = new anchor.BN(0.05 * LAMPORTS_PER_SOL); // 0.05 SOL
    
    await program.methods
      .withdraw(withdrawAmount)
      .accounts({
        vault: vaultKeypair.publicKey,
        vaultAuthority: vaultAuthority,
        user: wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();
    
    console.log(`Вывод ${withdrawAmount.toString()} лампортов (${withdrawAmount.toNumber() / LAMPORTS_PER_SOL} SOL) успешно выполнен`);
    
    // Проверка обновленного баланса
    const updatedVault = await program.account.vault.fetch(vaultKeypair.publicKey);
    console.log(`Обновленный баланс: ${updatedVault.balance.toString()} лампортов (${updatedVault.balance.toNumber() / LAMPORTS_PER_SOL} SOL)`);
    
  } catch (err) {
    console.error("Ошибка:", err);
  }
}

main();