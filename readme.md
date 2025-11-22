# CLMM Pool (Soroban Testnet)

Kontrak ini adalah prototipe **Concentrated Liquidity Market Maker (CLMM)** sederhana di jaringan **Stellar Soroban Testnet**.  
Fitur yang tersedia:

- Add Liquidity (dengan tick range)
- Remove Liquidity
- Swap Token A ↔ Token B
- Baca posisi liquidity
- Baca state pool

Kontrak ini cocok untuk testing UI dan demo Soroban.

---

## 📡 Network

- **Soroban RPC URL:** https://soroban-testnet.stellar.org  
- **Network Passphrase:** Test SDF Network ; September 2015  
- **Network:** Testnet

---

## 📌 Contract IDs

### CLMM Pool Contract
CCRCPFLRG3N3VTBDNCI4JSVC3RYKA5D7NZB2VTRCYK7WRL6LZ3FBDROZ

graphql
Salin kode

### Token A: XLM (native-wrapped Soroban token)
CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC

shell
Salin kode

### Token B: USDC Testnet
Soroban contract:
CBIELTK6YBZJU5UP2WWQEUCYKLPU6AUNZ2BQ4WWFEIE3USCIHMXQDAMA

yaml
Salin kode

Classic asset (jika perlu trustline):
- Code: USDC  
- Issuer: `GBBD47IF6LWK7P7MDEVSCWR7DPUWV3NY3DTQEVFL4NAT4AQH3ZLLFLA5`

---

## 🛠 Prerequisite

Sebelum menggunakan kontrak:

1. Install Stellar CLI  
2. Punya akun testnet (`freighter` atau CLI keypair)  
3. Punya:
   - XLM
   - USDC Testnet
4. Sudah **punya trustline USDC** (jika lewat Freighter)

---

# 📘 Cara Menggunakan Kontrak

Semua contoh menggunakan akun `alice`.

---

# 1️⃣ Cek Pool State

stellar contract invoke
--id CCRCPFLRG3N3VTBDNCI4JSVC3RYKA5D7NZB2VTRCYK7WRL6LZ3FBDROZ
--network testnet
-- get_pool_state

yaml
Salin kode

Output menampilkan:

- sqrt_price_x64  
- current_tick  
- liquidity  
- tick_spacing  
- token0 / token1  

---

# 2️⃣ Add Liquidity

Contoh:  
- Tick range: -10 → 10  
- Liquidity: 1,000,000  
- Token A: 5,000,000  
- Token B: 5,000,000  

stellar contract invoke
--id CCRCPFLRG3N3VTBDNCI4JSVC3RYKA5D7NZB2VTRCYK7WRL6LZ3FBDROZ
--network testnet
--source-account alice
-- add_liquidity
--owner alice
--lower -10
--upper 10
--liquidity 1000000
--amt_a 5000000
--amt_b 5000000

yaml
Salin kode

Jika sukses:
- Token A dan B akan dikirim ke kontrak
- Ticks akan diperbarui
- Posisi tersimpan untuk `alice`

---

# 3️⃣ Cek Posisi Liquidity

stellar contract invoke
--id CCRCPFLRG3N3VTBDNCI4JSVC3RYKA5D7NZB2VTRCYK7WRL6LZ3FBDROZ
--network testnet
-- get_position
--owner alice
--lower -10
--upper 10

yaml
Salin kode

Output:
- liquidity  
- token_a_amount  
- token_b_amount  

---

# 4️⃣ Remove Liquidity

Contoh: menarik 500,000 liquidity

stellar contract invoke
--id CCRCPFLRG3N3VTBDNCI4JSVC3RYKA5D7NZB2VTRCYK7WRL6LZ3FBDROZ
--network testnet
--source-account alice
-- remove_liquidity
--owner alice
--lower -10
--upper 10
--liquidity 500000

yaml
Salin kode

Kontrak mengembalikan:
- Token A
- Token B
- Mengurangi liquidity global dan posisi

---

# 5️⃣ Swap

### zero_for_one = true  
Token A → Token B  
(XLM → USDC)

### zero_for_one = false  
Token B → Token A  
(USDC → XLM)

Contoh swap 1000 native → USDC:

stellar contract invoke
--id CCRCPFLRG3N3VTBDNCI4JSVC3RYKA5D7NZB2VTRCYK7WRL6LZ3FBDROZ
--network testnet
--source-account alice
-- swap
--caller alice
--amount_specified 1000
--zero_for_one true
--sqrt_price_limit_x64 0

yaml
Salin kode

Output:
- amount_in  
- amount_out  
- current_tick  
- sqrt_price_x64  

---

# 🧪 Testing Tips
- Gunakan akun berbeda untuk swap & liquidity  
- UI dapat menggunakan RPC publik: https://soroban-testnet.stellar.org  
- Gunakan `get_pool_state` untuk sinkronisasi data UI  

---

# 📄 Lisensi
Kontrak ini open-source untuk pembelajaran & eksperimen CLMM di Soroban.

