//! NetChain Wallet CLI
//!
//! This binary shares transaction and wallet formats with the node via the `netchain` library crate.

use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use ed25519_dalek::SigningKey;
use netchain::rpc_types::{RpcRequest, RpcResponse};
use netchain::transaction::{SignedTransaction, Transaction};
use netchain::wallet::{self, Wallet, WalletFile};
use std::io::{self, Write};
use std::path::PathBuf;
use zeroize::Zeroizing;

#[derive(Parser)]
#[command(name = "netchain-wallet")]
#[command(about = "NetChain Wallet CLI - Manage wallets and send transactions")]
#[command(version = "0.2.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// RPC server URL
    #[arg(long, default_value = "http://127.0.0.1:8545")]
    rpc: String,

    /// Wallet directory
    #[arg(long)]
    wallet_dir: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new wallet (encrypted with password)
    New {
        /// Wallet name
        #[arg(short, long)]
        name: String,
    },

    /// List all wallets
    List,

    /// Get balance for an address
    Balance {
        /// Address to check (or wallet name)
        address: String,
    },

    /// Send tokens to another address
    Send {
        /// Sender wallet address (or name)
        #[arg(short, long)]
        from: String,

        /// Recipient address
        #[arg(short, long)]
        to: String,

        /// Amount to send
        #[arg(short, long)]
        amount: u64,

        /// Transaction fee
        #[arg(long, default_value = "1")]
        fee: u64,

        /// Optional memo
        #[arg(short, long)]
        memo: Option<String>,
    },

    /// Get blockchain status
    Status,

    /// Show wallet details
    Show {
        /// Wallet address (or name/prefix)
        address: String,
    },
}

fn get_wallet_dir(cli_dir: Option<PathBuf>) -> PathBuf {
    cli_dir.unwrap_or_else(wallet::default_wallet_dir)
}

fn prompt_password(prompt: &str) -> Result<Zeroizing<String>> {
    let mut password = Zeroizing::new(read_password(prompt)?);
    let trimmed_len = password.trim_end_matches(&['\n', '\r'][..]).len();
    password.truncate(trimmed_len);
    if password.is_empty() {
        return Err(anyhow!("Password cannot be empty"));
    }
    Ok(password)
}

#[cfg(unix)]
fn read_password(prompt: &str) -> Result<String> {
    use std::os::unix::io::AsRawFd;

    eprint!("{}", prompt);
    io::stderr().flush()?;

    let fd = io::stdin().as_raw_fd();

    unsafe {
        let mut term: libc::termios = std::mem::zeroed();
        if libc::tcgetattr(fd, &mut term) != 0 {
            return read_password_fallback();
        }

        let original = term;
        let mut modified = term;
        modified.c_lflag &= !libc::ECHO;

        if libc::tcsetattr(fd, libc::TCSANOW, &modified) != 0 {
            return read_password_fallback();
        }

        struct Restore {
            fd: i32,
            term: libc::termios,
        }
        impl Drop for Restore {
            fn drop(&mut self) {
                unsafe {
                    let _ = libc::tcsetattr(self.fd, libc::TCSANOW, &self.term);
                }
            }
        }

        let _restore = Restore { fd, term: original };

        let mut password = String::new();
        io::stdin().read_line(&mut password)?;
        eprintln!();
        Ok(password)
    }
}

#[cfg(not(unix))]
fn read_password(prompt: &str) -> Result<String> {
    eprint!("{}", prompt);
    io::stderr().flush()?;
    read_password_fallback()
}

fn read_password_fallback() -> Result<String> {
    let mut password = String::new();
    io::stdin().read_line(&mut password)?;
    Ok(password)
}

fn prompt_new_password() -> Result<Zeroizing<String>> {
    let p1 = prompt_password("Enter password for wallet encryption: ")?;
    let p2 = prompt_password("Confirm password: ")?;
    if p1.as_str() != p2.as_str() {
        return Err(anyhow!("Passwords do not match"));
    }
    Ok(p1)
}

fn list_wallets(wallet_dir: &PathBuf) -> Result<Vec<WalletFile>> {
    wallet::list_wallets(wallet_dir)
}

fn find_wallet(wallet_dir: &PathBuf, address_or_name: &str) -> Result<WalletFile> {
    let wallets = list_wallets(wallet_dir)?;

    // Try exact address match first
    if let Some(w) = wallets.iter().find(|w| w.address == address_or_name) {
        return Ok(w.clone());
    }

    // Try name match
    if let Some(w) = wallets.iter().find(|w| w.name == address_or_name) {
        return Ok(w.clone());
    }

    // Try partial address match
    if let Some(w) = wallets
        .iter()
        .find(|w| w.address.starts_with(address_or_name))
    {
        return Ok(w.clone());
    }

    Err(anyhow!("Wallet not found: {}", address_or_name))
}

fn load_signing_key(wallet_dir: &PathBuf, wallet: &WalletFile) -> Result<SigningKey> {
    let filepath = wallet_dir.join(format!("{}.json", wallet.address));
    let password = if wallet.version >= 2 {
        prompt_password("Enter wallet password: ")?
    } else {
        Zeroizing::new(String::new())
    };

    let loaded = Wallet::load_encrypted(&filepath, password.as_str())?;
    Ok(loaded.signing_key)
}

async fn rpc_call(rpc_url: &str, request: RpcRequest) -> Result<RpcResponse> {
    let client = reqwest::Client::new();

    let response = client
        .post(format!("{}/rpc", rpc_url))
        .json(&request)
        .send()
        .await?;

    let rpc_response: RpcResponse = response.json().await?;
    Ok(rpc_response)
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let wallet_dir = get_wallet_dir(cli.wallet_dir);

    match cli.command {
        Commands::New { name } => {
            let password = prompt_new_password()?;

            let wallet = Wallet::generate(name.clone());
            let filepath = wallet.save_encrypted(&wallet_dir, password.as_str())?;

            println!("Wallet created successfully!");
            println!("   Name:    {}", wallet.name);
            println!("   Address: {}", wallet.address);
            println!("   File:    {}", filepath.display());
            println!("   Format:  Encrypted (v2, AES-256-GCM + Argon2id)");
            println!("\n   Remember your password -- there is no recovery mechanism.");
        }

        Commands::List => {
            let wallets = list_wallets(&wallet_dir)?;

            if wallets.is_empty() {
                println!("No wallets found. Create one with: netchain-wallet new --name <name>");
                return Ok(());
            }

            println!("Wallets in {}:\n", wallet_dir.display());
            println!(
                "{:<15} {:<42} {:<5} {}",
                "NAME", "ADDRESS", "VER", "CREATED"
            );
            println!("{}", "-".repeat(80));

            for w in wallets {
                let created = w.created_at.split('T').next().unwrap_or(&w.created_at);
                let ver = format!("v{}", w.version);
                println!("{:<15} {:<42} {:<5} {}", w.name, w.address, ver, created);
            }
        }

        Commands::Balance { address } => {
            let addr = match find_wallet(&wallet_dir, &address) {
                Ok(w) => w.address,
                Err(_) => address,
            };

            let response = rpc_call(
                &cli.rpc,
                RpcRequest::GetBalance {
                    address: addr.clone(),
                },
            )
            .await?;

            match response {
                RpcResponse::Success { data } => {
                    let balance = data["balance"].as_u64().unwrap_or(0);
                    println!("Balance for {}:", addr);
                    println!("   {} tokens", balance);
                }
                RpcResponse::Error { message } => {
                    println!("Error: {}", message);
                }
            }
        }

        Commands::Send {
            from,
            to,
            amount,
            fee,
            memo,
        } => {
            let wallet_meta = find_wallet(&wallet_dir, &from)?;
            let signing_key = load_signing_key(&wallet_dir, &wallet_meta)?;

            let nonce_response = rpc_call(
                &cli.rpc,
                RpcRequest::GetNonce {
                    address: wallet_meta.address.clone(),
                },
            )
            .await?;

            let nonce = match nonce_response {
                RpcResponse::Success { data } => data["nonce"].as_u64().unwrap_or(0),
                RpcResponse::Error { message } => {
                    println!("Error getting nonce: {}", message);
                    return Ok(());
                }
            };

            let tx = Transaction::new(
                wallet_meta.address.clone(),
                to.clone(),
                amount,
                fee,
                nonce,
                memo,
            );
            let signed_tx = SignedTransaction::sign_with_keypair(&tx, &signing_key);

            let tx_json = serde_json::to_string(&signed_tx)?;
            let response = rpc_call(&cli.rpc, RpcRequest::SendTransaction { tx_json }).await?;

            match response {
                RpcResponse::Success { data } => {
                    let tx_hash = data["tx_hash"].as_str().unwrap_or("unknown");
                    println!("Transaction submitted!");
                    println!("   From:    {}", wallet_meta.address);
                    println!("   To:      {}", to);
                    println!("   Amount:  {}", amount);
                    println!("   Fee:     {}", fee);
                    println!("   TX Hash: {}", tx_hash);
                }
                RpcResponse::Error { message } => {
                    println!("Transaction failed: {}", message);
                }
            }
        }

        Commands::Status => {
            println!("Fetching blockchain status...\n");

            let chain_info = rpc_call(&cli.rpc, RpcRequest::GetChainInfo).await?;
            let mempool_size = rpc_call(&cli.rpc, RpcRequest::GetMempoolSize).await?;

            match chain_info {
                RpcResponse::Success { data } => {
                    println!("Blockchain Info:");
                    println!("   Height:       {}", data["height"]);
                    println!("   Latest Block: {}", data["latest_block_hash"]);
                    println!("   Genesis:      {}", data["genesis_hash"]);
                }
                RpcResponse::Error { message } => {
                    println!("Error: {}", message);
                    println!("\nMake sure the node is running with RPC enabled.");
                    return Ok(());
                }
            }

            if let RpcResponse::Success { data } = mempool_size {
                println!("\nMempool:");
                println!("   Pending TXs:  {}", data["size"]);
            }
        }

        Commands::Show { address } => {
            let wallet = find_wallet(&wallet_dir, &address)?;

            println!("Wallet Details:\n");
            println!("   Name:        {}", wallet.name);
            println!("   Address:     {}", wallet.address);
            println!(
                "   Format:      v{}{}",
                wallet.version,
                if wallet.version >= 2 {
                    " (encrypted)"
                } else {
                    " (legacy, UNENCRYPTED)"
                }
            );
            println!("   Created:     {}", wallet.created_at);

            // Try to get balance
            if let Ok(response) = rpc_call(
                &cli.rpc,
                RpcRequest::GetBalance {
                    address: wallet.address.clone(),
                },
            )
            .await
            {
                if let RpcResponse::Success { data } = response {
                    let balance = data["balance"].as_u64().unwrap_or(0);
                    println!("   Balance:     {} tokens", balance);
                }
            }
        }
    }

    Ok(())
}
