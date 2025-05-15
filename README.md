## Структура проекта
- Anchor.toml - файл конфигурации проекта
- programs/solana_deposit_app/src/lib.rs - смарт-контракт
- programs/solana_deposit_app/src/lib.rs - файл с кодом смарт-контракта
- programs/solana_deposit_app/Cargo.toml - конфигурация для Rust
- app/index.ts - клиентское приложение
- tests/solana-deposit-app.ts - набор тестов для проверки функциональности контракт

## Смарт-контракт
Основные функции смарт-контракта:
- initialize - создание нового хранилища депозитов
- deposit - внесение SOL на депозит
- withdraw - вывод SOL с депозита
- check_balance - проверка текущего баланса

Структура данных:
- Vault - структура для хранения информации о депозите:
  - owner - владелец депозита (Pubkey)
  - balance - текущий баланс в лампортах (1 SOL = 1,000,000,000 лампортов)
