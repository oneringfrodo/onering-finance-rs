import assert from "assert";
import * as anchor from "@project-serum/anchor";
import { web3, Program, BN } from "@project-serum/anchor";
import { TOKEN_PROGRAM_ID, Token } from "@solana/spl-token";
import { OneringFinance } from "../target/types/onering_finance";

// PDA seeds
const OUSD_MINT_AUTH_SEED = "or_ousd_mint_auth";
const STABLE_VAULT_SEED = "or_stable_vault";
const RESERVE_SEED = "or_reserve";

// main state & 1USD mint
const STATE_KEYPAIR = web3.Keypair.generate();
let ousdMint: Token;
let ousdMintAuthPda: web3.PublicKey, ousdMintAuthBump: number;

// market, stable mint, stable vault
const MARKET_KEYPAIR = web3.Keypair.generate();
const STABLE_MINT_AUTH_KEYPAIR = web3.Keypair.generate();
let stableMint: Token;
let stableVaultPda: web3.PublicKey, stableVaultBump: number;

// accounts
const FEE_PAYER_KEYPAIR = web3.Keypair.generate();
const ADMIN_KEYPAIR = web3.Keypair.generate();
const NEW_ADMIN_KEYPAIR = web3.Keypair.generate();
const USER_KEYPAIR = web3.Keypair.generate();

// amounts
const DEPOSIT_AMOUNT = new BN("100000000");

// reserve
let reservePda: web3.PublicKey, reserveBump: number;
let initializerStableToken: web3.PublicKey,
  initializerOusdToken: web3.PublicKey;

describe("onering-finance", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.Provider.env();
  anchor.setProvider(provider);

  // @ts-ignore
  const program = anchor.workspace.OneringFinance as Program<OneringFinance>;

  before(async () => {
    // airdrops 10 SOLs to fee payer
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(
        FEE_PAYER_KEYPAIR.publicKey,
        10 * web3.LAMPORTS_PER_SOL
      ),
      "confirmed"
    );

    // funds main accounts
    await provider.send(
      (() => {
        const tx = new web3.Transaction();
        tx.add(
          web3.SystemProgram.transfer({
            fromPubkey: FEE_PAYER_KEYPAIR.publicKey,
            toPubkey: ADMIN_KEYPAIR.publicKey,
            lamports: web3.LAMPORTS_PER_SOL,
          }),
          web3.SystemProgram.transfer({
            fromPubkey: FEE_PAYER_KEYPAIR.publicKey,
            toPubkey: NEW_ADMIN_KEYPAIR.publicKey,
            lamports: web3.LAMPORTS_PER_SOL,
          }),
          web3.SystemProgram.transfer({
            fromPubkey: FEE_PAYER_KEYPAIR.publicKey,
            toPubkey: USER_KEYPAIR.publicKey,
            lamports: web3.LAMPORTS_PER_SOL,
          })
        );
        return tx;
      })(),
      [FEE_PAYER_KEYPAIR]
    );

    // 1USD mint authority (PDA)
    [ousdMintAuthPda, ousdMintAuthBump] =
      await web3.PublicKey.findProgramAddress(
        [
          Buffer.from(anchor.utils.bytes.utf8.encode(OUSD_MINT_AUTH_SEED)),
          STATE_KEYPAIR.publicKey.toBuffer(),
        ],
        program.programId
      );

    // create 1USD token mint
    ousdMint = await Token.createMint(
      provider.connection,
      FEE_PAYER_KEYPAIR,
      ousdMintAuthPda,
      null,
      6,
      TOKEN_PROGRAM_ID
    );

    // initializer 1USD token ATA
    initializerOusdToken = await ousdMint.createAssociatedTokenAccount(
      USER_KEYPAIR.publicKey
    );

    // create stable token mint
    stableMint = await Token.createMint(
      provider.connection,
      FEE_PAYER_KEYPAIR,
      STABLE_MINT_AUTH_KEYPAIR.publicKey,
      null,
      6,
      TOKEN_PROGRAM_ID
    );

    // initializer stable token ATA
    initializerStableToken = await stableMint.createAssociatedTokenAccount(
      USER_KEYPAIR.publicKey
    );

    // mint 100 tokens to initializer stable token ATA
    await stableMint.mintTo(
      initializerStableToken,
      STABLE_MINT_AUTH_KEYPAIR.publicKey,
      [STABLE_MINT_AUTH_KEYPAIR],
      DEPOSIT_AMOUNT.toNumber()
    );

    // stable vault, authority itself (PDA)
    [stableVaultPda, stableVaultBump] = await web3.PublicKey.findProgramAddress(
      [
        stableMint.publicKey.toBuffer(),
        Buffer.from(anchor.utils.bytes.utf8.encode(STABLE_VAULT_SEED)),
        MARKET_KEYPAIR.publicKey.toBuffer(),
      ],
      program.programId
    );

    // reserve PDA
    [reservePda, reserveBump] = await web3.PublicKey.findProgramAddress(
      [
        USER_KEYPAIR.publicKey.toBuffer(),
        Buffer.from(anchor.utils.bytes.utf8.encode(RESERVE_SEED)),
        STATE_KEYPAIR.publicKey.toBuffer(),
      ],
      program.programId
    );
  });

  it("should create an admin", async () => {
    await program.rpc.createAdmin(
      { ousdMintAuthBump },
      {
        accounts: {
          admin: ADMIN_KEYPAIR.publicKey,
          ousdMint: ousdMint.publicKey,
          state: STATE_KEYPAIR.publicKey,
        },
        instructions: [
          await program.account.state.createInstruction(STATE_KEYPAIR),
        ],
        signers: [ADMIN_KEYPAIR, STATE_KEYPAIR],
      }
    );

    // asserts
    const state = await program.account.state.fetch(STATE_KEYPAIR.publicKey);
    assert.ok(state.admin.equals(ADMIN_KEYPAIR.publicKey));
    assert.ok(state.ousdMint.equals(ousdMint.publicKey));
    assert.ok(!state.emergencyFlag);
  });

  it("should create a market (stable token pool)", async () => {
    await program.rpc.createMarket(
      { stableVaultBump },
      {
        accounts: {
          admin: ADMIN_KEYPAIR.publicKey,
          stableMint: stableMint.publicKey,
          stableVault: stableVaultPda,
          market: MARKET_KEYPAIR.publicKey,
          state: STATE_KEYPAIR.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: web3.SystemProgram.programId,
          rent: web3.SYSVAR_RENT_PUBKEY,
        },
        instructions: [
          await program.account.market.createInstruction(MARKET_KEYPAIR),
        ],
        signers: [ADMIN_KEYPAIR, MARKET_KEYPAIR],
      }
    );

    // asserts
    const market = await program.account.market.fetch(MARKET_KEYPAIR.publicKey);
    assert.ok(market.stableMint.equals(stableMint.publicKey));
    assert.ok(market.stableVaultBump === stableVaultBump);
    assert.ok(market.withdrawalLiq.eq(new BN("0")));
    assert.ok(!market.lockFlag);
  });

  it("should mint 100 $1USD", async () => {
    await program.rpc.mint(
      { amount: DEPOSIT_AMOUNT },
      {
        accounts: {
          initializer: USER_KEYPAIR.publicKey,
          stableMint: stableMint.publicKey,
          stableVault: stableVaultPda,
          initializerStableToken,
          ousdMint: ousdMint.publicKey,
          ousdMintAuth: ousdMintAuthPda,
          initializerOusdToken,
          market: MARKET_KEYPAIR.publicKey,
          state: STATE_KEYPAIR.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        },
        signers: [USER_KEYPAIR],
      }
    );

    // asserts
    const stableVaultAccount = await stableMint.getAccountInfo(stableVaultPda);
    assert.ok(stableVaultAccount.amount.eq(DEPOSIT_AMOUNT));
    const initializerStableTokenAccount = await stableMint.getAccountInfo(
      initializerStableToken
    );
    assert.ok(initializerStableTokenAccount.amount.eq(new BN("0")));
    const initializerOusdTokenAccount = await ousdMint.getAccountInfo(
      initializerOusdToken
    );
    assert.ok(initializerOusdTokenAccount.amount.eq(DEPOSIT_AMOUNT));
  });

  it("should deposit (old stake) 100 $1USD", async () => {
    await program.rpc.deposit(
      { amount: DEPOSIT_AMOUNT },
      {
        accounts: {
          initializer: USER_KEYPAIR.publicKey,
          ousdMint: ousdMint.publicKey,
          initializerOusdToken,
          reserve: reservePda,
          state: STATE_KEYPAIR.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        },
        instructions: [
          // create deposit reserve PDA
          await program.instruction.createReserve(
            { nonce: reserveBump },
            {
              accounts: {
                initializer: USER_KEYPAIR.publicKey,
                reserve: reservePda,
                state: STATE_KEYPAIR.publicKey,
                systemProgram: web3.SystemProgram.programId,
                rent: web3.SYSVAR_RENT_PUBKEY,
              },
              signers: [USER_KEYPAIR],
            }
          ),
        ],
        signers: [USER_KEYPAIR],
      }
    );

    // asserts
    const reserve = await program.account.reserve.fetch(reservePda);
    assert.ok(reserve.nonce === reserveBump);
    assert.ok(reserve.depositAmount.eq(DEPOSIT_AMOUNT));
    assert.ok(!reserve.freezeFlag);
    const state = await program.account.state.fetch(STATE_KEYPAIR.publicKey);
    assert.ok(state.depositAmount.eq(DEPOSIT_AMOUNT));
    const initializerOusdTokenAccount = await ousdMint.getAccountInfo(
      initializerOusdToken
    );
    assert.ok(initializerOusdTokenAccount.amount.eq(new BN("0")));
  });

  it("should withdraw (old unstake) 100 $1USD", async () => {
    await program.rpc.withdraw(
      { amount: DEPOSIT_AMOUNT },
      {
        accounts: {
          initializer: USER_KEYPAIR.publicKey,
          ousdMint: ousdMint.publicKey,
          ousdMintAuth: ousdMintAuthPda,
          initializerOusdToken,
          reserve: reservePda,
          state: STATE_KEYPAIR.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        },
        signers: [USER_KEYPAIR],
      }
    );

    // asserts
    const reserve = await program.account.reserve.fetch(reservePda);
    assert.ok(reserve.nonce === reserveBump);
    assert.ok(reserve.depositAmount.eq(new BN("0")));
    assert.ok(!reserve.freezeFlag);
    const state = await program.account.state.fetch(STATE_KEYPAIR.publicKey);
    assert.ok(state.depositAmount.eq(new BN("0")));
    const initializerOusdTokenAccount = await ousdMint.getAccountInfo(
      initializerOusdToken
    );
    assert.ok(initializerOusdTokenAccount.amount.eq(DEPOSIT_AMOUNT));
  });

  it("should redeem 100 $1USD", async () => {
    await program.rpc.redeem(
      { amount: DEPOSIT_AMOUNT },
      {
        accounts: {
          initializer: USER_KEYPAIR.publicKey,
          stableMint: stableMint.publicKey,
          stableVault: stableVaultPda,
          initializerStableToken,
          ousdMint: ousdMint.publicKey,
          initializerOusdToken,
          market: MARKET_KEYPAIR.publicKey,
          state: STATE_KEYPAIR.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        },
        signers: [USER_KEYPAIR],
      }
    );

    // asserts
    const stableVaultAccount = await stableMint.getAccountInfo(stableVaultPda);
    assert.ok(stableVaultAccount.amount.eq(new BN("0")));
    const initializerStableTokenAccount = await stableMint.getAccountInfo(
      initializerStableToken
    );
    assert.ok(initializerStableTokenAccount.amount.eq(DEPOSIT_AMOUNT));
    const initializerOusdTokenAccount = await ousdMint.getAccountInfo(
      initializerOusdToken
    );
    assert.ok(initializerOusdTokenAccount.amount.eq(new BN("0")));
  });

  it("should mint & deposit (old stake) 100 $1USD", async () => {
    await program.rpc.mintAndDeposit(
      { amount: DEPOSIT_AMOUNT },
      {
        accounts: {
          initializer: USER_KEYPAIR.publicKey,
          stableMint: stableMint.publicKey,
          stableVault: stableVaultPda,
          initializerStableToken,
          reserve: reservePda,
          market: MARKET_KEYPAIR.publicKey,
          state: STATE_KEYPAIR.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        },
        signers: [USER_KEYPAIR],
      }
    );

    // asserts
    const reserve = await program.account.reserve.fetch(reservePda);
    assert.ok(reserve.nonce === reserveBump);
    assert.ok(reserve.depositAmount.eq(DEPOSIT_AMOUNT));
    assert.ok(!reserve.freezeFlag);
    const state = await program.account.state.fetch(STATE_KEYPAIR.publicKey);
    assert.ok(state.depositAmount.eq(DEPOSIT_AMOUNT));
    const stableVaultAccount = await stableMint.getAccountInfo(stableVaultPda);
    assert.ok(stableVaultAccount.amount.eq(DEPOSIT_AMOUNT));
    const initializerStableTokenAccount = await stableMint.getAccountInfo(
      initializerStableToken
    );
    assert.ok(initializerStableTokenAccount.amount.eq(new BN("0")));
    const initializerOusdTokenAccount = await ousdMint.getAccountInfo(
      initializerOusdToken
    );
    assert.ok(initializerOusdTokenAccount.amount.eq(new BN("0")));
  });
});
