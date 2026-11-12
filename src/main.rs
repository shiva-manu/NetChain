// src/main.rs
//! NetChain node.
//!
//! ```text
//! netchain                 run the node (same as `netchain run`)
//! netchain id              print this node's address / peer id (creates the node key)
//! netchain keygen          print a new node key (for NETCHAIN_NODE_KEY on a PaaS)
//! netchain genesis ...     write a genesis file for a new network
//! ```

use anyhow::{Context, Result};
use chrono::Utc;
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::time::interval;
use tracing::{debug, error, info, warn};
use tracing_subscriber::EnvFilter;

use netchain::block::Block;
use netchain::chain_service::{ChainService, GossipImport};
use netchain::config::AppConfig;
use netchain::engine;
use netchain::genesis::{GenesisAllocation, GenesisSpec, GenesisValidator};
use netchain::http::{self, Gateway};
use netchain::monitoring::MonitoringState;
use netchain::node_key;
use netchain::p2p::{P2PConfig, P2PEvent, P2PHandle, P2PService};
use netchain::rpc::{Faucet, NodeInfo, RpcState};
use netchain::storage::Storage;
use netchain::transaction::{SignedTransaction, Transaction};
use netchain::websocket::{self, WsEvent};

/// Delay before producing blocks, so a restarted node syncs first instead of forking.
const STARTUP_SYNC_GRACE: Duration = Duration::from_secs(6);
/// How often to ask peers for blocks we may have missed.
const SYNC_POLL_INTERVAL: Duration = Duration::from_secs(10);
/// Upper bound for one sync response (serialized bytes).
const MAX_SYNC_RESPONSE_BYTES: usize = 8 * 1024 * 1024;

#[derive(Parser)]
#[command(
    name = "netchain",
    version,
    about = "NetChain node (hybrid Proof-of-Internet L1)"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Run the node (default).
    Run,
    /// Print this node's on-chain address and libp2p peer id (creates the node key if missing).
    Id,
    /// Generate a new node key and print it with its address and peer id. Use the secret as
    /// `NETCHAIN_NODE_KEY` where the data directory is not the place to keep keys.
    Keygen,
    /// Write a genesis file for a new network.
    Genesis {
        /// Network name.
        #[arg(long, default_value = "netchain-testnet")]
        chain_id: String,
        /// Initial validator as ADDRESS:STAKE (repeat for several validators).
        #[arg(long = "validator", required = true)]
        validators: Vec<String>,
        /// Initial balance as ADDRESS:AMOUNT (repeat; e.g. the faucet account).
        #[arg(long = "alloc")]
        allocations: Vec<String>,
        /// Genesis time in unix seconds (default: now).
        #[arg(long)]
        timestamp: Option<i64>,
        /// Block interval in seconds.
        #[arg(long)]
        block_interval: Option<u64>,
        /// Output path, or `-` to print compact JSON to stdout.
        #[arg(long, default_value = "genesis.json")]
        out: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = AppConfig::load()?;

    match cli.command.unwrap_or(Command::Run) {
        Command::Id => print_identity(&config),
        Command::Keygen => keygen(),
        Command::Genesis {
            chain_id,
            validators,
            allocations,
            timestamp,
            block_interval,
            out,
        } => write_genesis(
            &config,
            chain_id,
            validators,
            allocations,
            timestamp,
            block_interval,
            &out,
        ),
        Command::Run => run_node(config).await,
    }
}

/// The node key: `NETCHAIN_NODE_KEY` if set, otherwise `<data_dir>/node_key` (created once).
fn resolve_node_key(config: &AppConfig) -> Result<ed25519_dalek::SigningKey> {
    match &config.node.node_key {
        Some(secret) => node_key::from_hex(&secret.0).context("invalid NETCHAIN_NODE_KEY"),
        None => node_key::load_or_create(&node_key::node_key_path(&config.data_dir())),
    }
}

fn keygen() -> Result<()> {
    let key = netchain::transaction::generate_ed25519_keypair();
    let peer_id = libp2p::PeerId::from(node_key::libp2p_keypair(&key)?.public());
    println!("secret:  {}", hex::encode(key.to_bytes()));
    println!("address: {}", node_key::address_of(&key));
    println!("peer_id: {peer_id}");
    eprintln!("Keep the secret private: it controls this validator's stake and rewards.");
    Ok(())
}

fn print_identity(config: &AppConfig) -> Result<()> {
    let key = resolve_node_key(config)?;
    let keypair = node_key::libp2p_keypair(&key)?;
    let peer_id = libp2p::PeerId::from(keypair.public());
    println!("address: {}", node_key::address_of(&key));
    println!("peer_id: {peer_id}");
    println!(
        "bootnode multiaddr: /ip4/<PUBLIC_IP>/tcp/{}/p2p/{peer_id}",
        config.node.p2p_port
    );
    Ok(())
}

fn parse_pair(s: &str, what: &str) -> Result<(String, u64)> {
    let (addr, amount) = s
        .split_once(':')
        .with_context(|| format!("{what} must be ADDRESS:AMOUNT, got {s:?}"))?;
    let amount = amount
        .parse()
        .with_context(|| format!("invalid amount in {s:?}"))?;
    Ok((addr.trim().to_lowercase(), amount))
}

fn write_genesis(
    config: &AppConfig,
    chain_id: String,
    validators: Vec<String>,
    allocations: Vec<String>,
    timestamp: Option<i64>,
    block_interval: Option<u64>,
    out: &Path,
) -> Result<()> {
    let mut params = config.dev_chain_params();
    if let Some(interval) = block_interval {
        params.block_interval_secs = interval.max(1);
    }
    let spec = GenesisSpec {
        chain_id,
        timestamp: timestamp.unwrap_or_else(|| Utc::now().timestamp()),
        validators: validators
            .iter()
            .map(|v| {
                parse_pair(v, "--validator")
                    .map(|(address, stake)| GenesisValidator { address, stake })
            })
            .collect::<Result<_>>()?,
        allocations: allocations
            .iter()
            .map(|a| {
                parse_pair(a, "--alloc")
                    .map(|(address, balance)| GenesisAllocation { address, balance })
            })
            .collect::<Result<_>>()?,
        params,
    };
    spec.validate()?;
    if out == Path::new("-") {
        // Compact single-line JSON, e.g. for a NETCHAIN_GENESIS_JSON variable.
        println!("{}", serde_json::to_string(&spec)?);
        eprintln!("genesis hash {}", spec.genesis_block().hash);
        return Ok(());
    }
    spec.save(out)?;
    println!(
        "wrote {} (genesis hash {})",
        out.display(),
        spec.genesis_block().hash
    );
    Ok(())
}

/// Genesis sources, in order: inline JSON, file, a cached copy in the data dir, a URL
/// (downloaded once and cached), or a new single-validator dev network.
async fn load_genesis(config: &AppConfig, node_address: &str) -> Result<GenesisSpec> {
    if let Some(json) = &config.node.genesis_json {
        let spec = GenesisSpec::from_json(json).context("invalid NETCHAIN_GENESIS_JSON")?;
        info!(chain_id = %spec.chain_id, "loaded genesis from NETCHAIN_GENESIS_JSON");
        return Ok(spec);
    }
    if let Some(path) = &config.node.genesis_file {
        let spec = GenesisSpec::load(Path::new(path))?;
        info!(chain_id = %spec.chain_id, file = %path, "loaded genesis");
        return Ok(spec);
    }
    let path = config.data_dir().join("genesis.json");
    if path.exists() {
        return GenesisSpec::load(&path);
    }
    if let Some(url) = &config.node.genesis_url {
        let body = reqwest::get(url)
            .await
            .and_then(|r| r.error_for_status())
            .with_context(|| format!("failed to download genesis from {url}"))?
            .text()
            .await?;
        let spec = GenesisSpec::from_json(&body)
            .with_context(|| format!("invalid genesis served by {url}"))?;
        spec.save(&path)?;
        info!(chain_id = %spec.chain_id, %url, "downloaded genesis");
        return Ok(spec);
    }
    let spec = GenesisSpec::dev(
        node_address,
        Utc::now().timestamp(),
        config.dev_chain_params(),
    );
    spec.save(&path)?;
    warn!(
        file = %path.display(),
        "no genesis configured: created a single-validator development network owned by this node"
    );
    Ok(spec)
}

async fn run_node(config: AppConfig) -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new(config.node.log_level.clone())),
        )
        .with_target(false)
        .with_ansi(std::io::IsTerminal::is_terminal(&std::io::stdout()))
        .init();

    let data_dir = config.data_dir();
    std::fs::create_dir_all(&data_dir)
        .with_context(|| format!("failed to create data dir {}", data_dir.display()))?;

    if std::env::var_os("RAILWAY_SERVICE_ID").is_some()
        && std::env::var_os("RAILWAY_VOLUME_MOUNT_PATH").is_none()
    {
        warn!(
            "running on Railway without a volume: chain data (and the node key, unless \
             NETCHAIN_NODE_KEY is set) will be lost on every redeploy. Attach a volume at {}",
            data_dir.display()
        );
    }

    // Identity
    let node_key = resolve_node_key(&config)?;
    let node_address = node_key::address_of(&node_key);
    let libp2p_key = node_key::libp2p_keypair(&node_key)?;

    // Chain
    let genesis = load_genesis(&config, &node_address).await?;
    let storage = Arc::new(Storage::open(&data_dir.join("db"))?);
    let ws_tx = websocket::create_event_channel();
    let chain = Arc::new(ChainService::load(
        genesis.clone(),
        storage.clone(),
        ws_tx.clone(),
    )?);
    let genesis_hash = chain.blockchain.lock().await.genesis().hash.clone();

    // Networking
    let bootnodes = config
        .node
        .bootnodes
        .iter()
        .map(|a| {
            a.parse()
                .with_context(|| format!("invalid bootnode multiaddr {a:?}"))
        })
        .collect::<Result<Vec<_>>>()?;
    let external_addrs = config
        .node
        .external_addrs
        .iter()
        .map(|a| {
            a.parse()
                .with_context(|| format!("invalid external multiaddr {a:?}"))
        })
        .collect::<Result<Vec<_>>>()?;
    let (mut p2p_service, p2p, p2p_commands) = P2PService::new(P2PConfig {
        port: config.node.p2p_port,
        keypair: libp2p_key,
        bootnodes,
        external_addrs,
        enable_mdns: config.node.enable_mdns,
        network_id: genesis_hash.clone(),
    })?;
    let (p2p_events_tx, mut p2p_events) = mpsc::channel(1024);
    tokio::spawn(async move { p2p_service.run(p2p_events_tx, p2p_commands).await });

    info!(
        chain_id = %genesis.chain_id,
        genesis = %genesis_hash,
        address = %node_address,
        peer_id = %p2p.local_peer_id(),
        height = chain.height().await,
        "NetChain node started"
    );
    {
        let state = chain.state.lock().await;
        if engine::active_validators(&state).contains(&node_address) {
            info!("this node is an active validator");
        } else {
            info!("this node is not a validator (stake NCN from its address to become one)");
        }
    }

    // Faucet
    let faucet = if config.faucet.enabled {
        let key = match &config.faucet.key_file {
            Some(path) => node_key::load(Path::new(path))?,
            None => node_key.clone(),
        };
        let faucet = Faucet::new(
            key,
            config.faucet.amount,
            config.faucet.fee,
            config.faucet.cooldown_secs,
            config.faucet.max_per_minute,
        );
        let balance = chain.state.lock().await.get_balance(&faucet.address);
        if balance == 0 {
            info!(address = %faucet.address, "faucet has no funds on this network; requests will fail");
        } else {
            info!(address = %faucet.address, balance, "faucet enabled");
        }
        Some(faucet)
    } else {
        None
    };

    // HTTP gateway: RPC, REST, monitoring, genesis and WebSocket on one port.
    let gateway = Arc::new(Gateway {
        rpc: Arc::new(RpcState {
            blockchain: chain.blockchain.clone(),
            state: chain.state.clone(),
            mempool: chain.mempool.clone(),
            p2p: p2p.clone(),
            tx_index: chain.tx_index.clone(),
            faucet,
            node_info: NodeInfo {
                chain_id: genesis.chain_id.clone(),
                genesis_hash: genesis_hash.clone(),
                node_address: node_address.clone(),
                peer_id: p2p.local_peer_id().to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        }),
        monitoring: Arc::new(MonitoringState {
            blockchain: chain.blockchain.clone(),
            mempool: chain.mempool.clone(),
            p2p_shared: p2p.shared_state(),
            state: chain.state.clone(),
            started_at: Instant::now(),
            chain_id: genesis.chain_id.clone(),
            node_address: node_address.clone(),
            peer_id: p2p.local_peer_id().to_string(),
        }),
        ws_tx: ws_tx.clone(),
        genesis_json: serde_json::to_string_pretty(&genesis)?,
    });
    let (http_bind, http_port) = (config.http.bind_addr.clone(), config.http.port);
    tokio::spawn(async move {
        if let Err(e) = http::serve(gateway, &http_bind, http_port).await {
            error!(error = %e, "http gateway stopped");
        }
    });

    // Block production: try every half second; the engine decides whether this node owns
    // the current slot.
    {
        let chain = chain.clone();
        let p2p = p2p.clone();
        let key = node_key.clone();
        let started = Instant::now();
        tokio::spawn(async move {
            let mut tick = interval(Duration::from_millis(500));
            loop {
                tick.tick().await;
                if started.elapsed() < STARTUP_SYNC_GRACE {
                    continue;
                }
                if let Some(block) = chain.try_produce(&key, Utc::now()).await {
                    match serde_json::to_string(&block) {
                        Ok(json) => p2p.publish_block(json),
                        Err(e) => error!(error = %e, "failed to encode block"),
                    }
                }
            }
        });
    }

    // Mempool expiry.
    {
        let chain = chain.clone();
        let ttl = config.producer.mempool_ttl_secs;
        tokio::spawn(async move {
            let mut tick = interval(Duration::from_secs(30));
            loop {
                tick.tick().await;
                let now = Utc::now().timestamp().max(0) as u64;
                let expired = chain.mempool.lock().await.expire_old(now, ttl);
                if expired > 0 {
                    info!(expired, "expired stale mempool transactions");
                }
            }
        });
    }

    // Periodic catch-up sync with all peers.
    {
        let chain = chain.clone();
        let p2p = p2p.clone();
        tokio::spawn(async move {
            let mut tick = interval(SYNC_POLL_INTERVAL);
            loop {
                tick.tick().await;
                if p2p.shared_state().get_peer_count() > 0 {
                    p2p.request_sync(None, chain.height().await + 1);
                }
            }
        });
    }

    // Proof of Internet: validators periodically probe their validator peers.
    {
        let chain = chain.clone();
        let p2p = p2p.clone();
        let me = node_address.clone();
        let every = Duration::from_secs(config.poi.attest_interval_secs.max(10));
        let probe_bytes = config.poi.probe_bytes;
        tokio::spawn(async move {
            let mut tick = interval(every);
            tick.tick().await; // let connections settle first
            loop {
                tick.tick().await;
                let active = engine::active_validators(&*chain.state.lock().await);
                if !active.contains(&me) {
                    continue;
                }
                for peer in p2p.shared_state().connected_peers() {
                    match node_key::address_from_peer_id(&peer) {
                        Some(addr) if addr != me && active.contains(&addr) => {
                            p2p.probe(peer, probe_bytes)
                        }
                        _ => {}
                    }
                }
            }
        });
    }

    // Automatic unjail: if this node's validator was jailed (e.g. it was offline), ask to
    // rejoin as soon as it is back and synced.
    {
        let chain = chain.clone();
        let p2p = p2p.clone();
        let key = node_key.clone();
        let me = node_address.clone();
        let fee = config.poi.attestation_fee.max(1);
        tokio::spawn(async move {
            let mut tick = interval(Duration::from_secs(15));
            tick.tick().await;
            loop {
                tick.tick().await;
                let signed = {
                    let state = chain.state.lock().await;
                    if !state.is_jailed(&me) {
                        continue;
                    }
                    let mut mempool = chain.mempool.lock().await;
                    if mempool.pending_count(&me) > 0 {
                        continue;
                    }
                    let tx = Transaction::unjail(me.clone(), fee, state.get_nonce(&me));
                    let signed = SignedTransaction::sign_with_keypair(&tx, &key);
                    if let Err(e) = mempool.add_transaction(signed.clone(), &state) {
                        warn!(error = ?e, "could not submit unjail transaction");
                        continue;
                    }
                    signed
                };
                info!("validator is jailed; submitted unjail transaction");
                if let Ok(json) = serde_json::to_string(&signed) {
                    p2p.publish_transaction(json);
                }
            }
        });
    }

    // Main event loop.
    let attestation_fee = config.poi.attestation_fee;
    loop {
        tokio::select! {
            maybe_event = p2p_events.recv() => {
                let Some(event) = maybe_event else { break };
                handle_p2p_event(event, &chain, &p2p, &node_key, &node_address, attestation_fee, &ws_tx).await;
            }
            _ = tokio::signal::ctrl_c() => {
                info!("shutting down");
                break;
            }
        }
    }
    if let Err(e) = storage.flush() {
        warn!(error = %e, "failed to flush storage");
    }
    Ok(())
}

async fn handle_p2p_event(
    event: P2PEvent,
    chain: &Arc<ChainService>,
    p2p: &P2PHandle,
    node_key: &ed25519_dalek::SigningKey,
    node_address: &str,
    attestation_fee: u64,
    ws_tx: &tokio::sync::broadcast::Sender<WsEvent>,
) {
    match event {
        P2PEvent::Block { json, source } => {
            let block: Block = match serde_json::from_str(&json) {
                Ok(b) => b,
                Err(e) => {
                    debug!(error = %e, "undecodable block from gossip");
                    return;
                }
            };
            match chain.import_gossip_block(block).await {
                GossipImport::NeedSync { from_height } => {
                    p2p.request_sync(Some(source), from_height)
                }
                GossipImport::Rejected(e) => {
                    warn!(error = %e, peer = %source, "rejected gossiped block")
                }
                _ => {}
            }
        }

        P2PEvent::Transaction(json) => {
            let Ok(tx) = serde_json::from_str::<SignedTransaction>(&json) else {
                return;
            };
            let accepted = {
                let state = chain.state.lock().await;
                let mut mempool = chain.mempool.lock().await;
                mempool.add_transaction(tx.clone(), &state).is_ok()
            };
            if accepted {
                let _ = ws_tx.send(WsEvent::NewTransaction {
                    tx_hash: tx.tx_hash_hex(),
                    sender: tx.tx.sender.clone(),
                    receiver: tx.tx.receiver.clone(),
                    amount: tx.tx.amount,
                    tx_type: format!("{:?}", tx.tx.tx_type),
                });
            }
        }

        P2PEvent::SyncRequest {
            from_height,
            channel,
            ..
        } => {
            let mut blocks = chain.blocks_for_sync(from_height.max(1)).await;
            let mut total = 0usize;
            let keep = blocks
                .iter()
                .take_while(|b| {
                    total += serde_json::to_vec(b).map(|v| v.len()).unwrap_or(0);
                    total <= MAX_SYNC_RESPONSE_BYTES
                })
                .count()
                .max(1)
                .min(blocks.len());
            blocks.truncate(keep);
            p2p.respond_sync(channel, blocks);
        }

        P2PEvent::SyncBlocks { peer, blocks } => {
            if blocks.is_empty() {
                return;
            }
            let outcome = chain.import_sync_blocks(blocks).await;
            if let Some(from) = outcome.request_from {
                p2p.request_sync(Some(peer), from);
            }
        }

        P2PEvent::ProbeResult {
            peer,
            bytes,
            elapsed,
        } => {
            let Some(subject) = netchain::node_key::address_from_peer_id(&peer) else {
                return;
            };
            let elapsed_ms = elapsed.as_millis().max(1) as u64;
            // kbps = bits / ms
            let throughput_kbps = (bytes as u64 * 8) / elapsed_ms;
            let rtt_ms = p2p
                .shared_state()
                .median_rtt(&peer)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(elapsed_ms);
            submit_attestation(
                chain,
                p2p,
                node_key,
                node_address,
                attestation_fee,
                subject,
                rtt_ms,
                throughput_kbps,
            )
            .await;
        }

        P2PEvent::PeerConnected(peer) => {
            p2p.request_sync(Some(peer), chain.height().await + 1);
        }

        P2PEvent::PeerDisconnected(_) => {}
    }
}

#[allow(clippy::too_many_arguments)]
async fn submit_attestation(
    chain: &Arc<ChainService>,
    p2p: &P2PHandle,
    node_key: &ed25519_dalek::SigningKey,
    node_address: &str,
    fee: u64,
    subject: String,
    rtt_ms: u64,
    throughput_kbps: u64,
) {
    let signed = {
        let state = chain.state.lock().await;
        let mut mempool = chain.mempool.lock().await;
        let nonce = state.get_nonce(node_address) + mempool.pending_count(node_address) as u64;
        let tx = Transaction::attest_metrics(
            node_address.to_string(),
            fee,
            nonce,
            subject.clone(),
            rtt_ms,
            throughput_kbps,
        );
        let signed = SignedTransaction::sign_with_keypair(&tx, node_key);
        if let Err(e) = mempool.add_transaction(signed.clone(), &state) {
            debug!(error = ?e, "attestation not accepted");
            return;
        }
        signed
    };
    info!(subject = %&subject[..10], rtt_ms, throughput_kbps, "submitted PoI attestation");
    if let Ok(json) = serde_json::to_string(&signed) {
        p2p.publish_transaction(json);
    }
}
