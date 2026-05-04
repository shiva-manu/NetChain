// src/main.rs

use anyhow::Result;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;
use tokio::sync::mpsc;
use tokio::time::{interval, sleep, Duration};
use tracing::{debug, error, info, warn};
use tracing_subscriber::EnvFilter;

use netchain::anti_gaming::{self, AntiGamingService};
use netchain::block::Block;
use netchain::blockchain::Blockchain;
use netchain::config::AppConfig;
use netchain::dht::{DhtService, DhtConfig, DhtEvent};
use netchain::epoch_manager::{EpochConfig, EpochManager};
use netchain::measurement::MeasurementService;
use netchain::mempool::Mempool;
use netchain::metric_challenge::MetricChallengeService;
use netchain::metrics_aggregator::{Attestation, MetricsAggregator};
use netchain::monitoring::{start_monitoring_server, MonitoringState};
use netchain::p2p::{NetworkMessage, P2PEvent, P2PService};
use netchain::producer::{BlockProducer, ProducerConfig};
use netchain::rpc::{start_rpc_server, RpcState};
use netchain::state::SlashReason;
use netchain::state::State;
use netchain::state::StateEvent;
use netchain::storage::Storage;
use netchain::transaction::SignedTransaction;
use netchain::websocket::{self, start_ws_server, WsEvent};

/// Convert a `SlashReason` to a human-readable string for WebSocket events.
fn slash_reason_label(reason: &SlashReason) -> &'static str {
    match reason {
        SlashReason::InvalidBlockProposal => "InvalidBlockProposal",
        SlashReason::MetricFraud => "MetricFraud",
        SlashReason::MissedBlock => "MissedBlock",
    }
}

fn slash_severity(reason: &SlashReason) -> f64 {
    match reason {
        SlashReason::InvalidBlockProposal => 0.35,
        SlashReason::MetricFraud => 0.25,
        SlashReason::MissedBlock => 0.10,
    }
}

async fn sync_validator_metrics_from_aggregator(
    producer: &Arc<Mutex<BlockProducer>>,
    aggregator: &Arc<Mutex<MetricsAggregator>>,
    state: &Arc<Mutex<State>>,
    node_id: &str,
) {
    let (consensus_metrics, slashing_penalty) = {
        let agg = aggregator.lock().await;
        let consensus_metrics = agg.get_consensus_node_metrics(node_id);
        drop(agg);

        let state_guard = state.lock().await;
        let slashing_penalty = state_guard.slashing_penalty_for(node_id);

        (consensus_metrics, slashing_penalty)
    };

    let mut metrics = consensus_metrics
        .unwrap_or_else(|| BlockProducer::default_node_metrics(node_id.to_string()));
    metrics.slashing_penalty = slashing_penalty;

    let mut producer_guard = producer.lock().await;
    if producer_guard.get_node_metrics(node_id).is_some() {
        producer_guard.update_peer_metrics(node_id, metrics);
    } else {
        producer_guard.register_peer(node_id.to_string(), metrics);
    }
}

async fn slash_validator(
    state: &Arc<Mutex<State>>,
    producer: &Arc<Mutex<BlockProducer>>,
    storage: &Arc<Storage>,
    ws_tx: &tokio::sync::broadcast::Sender<WsEvent>,
    node_id: &str,
    reason: SlashReason,
) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let (burned, remaining) = {
        let mut state_guard = state.lock().await;
        let burned = state_guard.slash_stake(node_id, reason.clone(), now);
        let remaining = state_guard.get_staked_balance(node_id);

        if burned > 0 {
            if let Err(e) = storage.save_state(&state_guard) {
                warn!(error = %e, "failed to persist state after slashing");
            }
        }

        (burned, remaining)
    };

    if burned > 0 {
        let mut producer_guard = producer.lock().await;
        producer_guard.penalize_validator(node_id, slash_severity(&reason));

        warn!(
            node_id,
            burned,
            reason = slash_reason_label(&reason),
            "validator slashed"
        );
        let _ = ws_tx.send(WsEvent::ValidatorSlashed {
            validator: node_id.to_string(),
            reason: slash_reason_label(&reason).to_string(),
            amount_burned: burned,
            remaining_stake: remaining,
        });
    }
}

/// Convert a `StateEvent` into a `WsEvent` for WebSocket broadcasting.
#[allow(dead_code)]
fn state_event_to_ws(event: &StateEvent) -> WsEvent {
    match event {
        StateEvent::ProposalCreated {
            proposal_id,
            title,
            proposer: _,
        } => WsEvent::ProposalUpdate {
            proposal_id: *proposal_id,
            title: title.clone(),
            status: "Created".to_string(),
            yes_votes: 0,
            no_votes: 0,
        },
        StateEvent::VoteCast {
            proposal_id,
            title,
            voter: _,
            support: _,
            yes_votes,
            no_votes,
        } => WsEvent::ProposalUpdate {
            proposal_id: *proposal_id,
            title: title.clone(),
            status: "VoteReceived".to_string(),
            yes_votes: *yes_votes,
            no_votes: *no_votes,
        },
    }
}

/// Parse chain sync blocks strictly.
///
/// If any block fails to deserialize, reject the entire response instead of
/// silently dropping the malformed entry and syncing a partial chain.
fn parse_chain_sync_blocks(blocks: &[String]) -> Result<Vec<Block>> {
    let mut parsed_blocks = Vec::with_capacity(blocks.len());

    for (index, block_json) in blocks.iter().enumerate() {
        let block = serde_json::from_str(block_json).map_err(|e| {
            anyhow::anyhow!(
                "invalid block at position {} in sync response: {}",
                index,
                e
            )
        })?;
        parsed_blocks.push(block);
    }

    Ok(parsed_blocks)
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = AppConfig::load()?;

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new(config.node.log_level.clone())),
        )
        .with_target(false)
        .init();

    info!("starting NetChain node");

    // Initialize persistent storage
    let data_dir = config.data_dir();

    info!(path = %data_dir.display(), "using data directory");
    let storage = Arc::new(Storage::open(&data_dir)?);

    // Load or create blockchain
    let blockchain = {
        let blocks = storage.load_all_blocks()?;
        if blocks.is_empty() {
            info!("creating new blockchain with genesis block");
            Arc::new(Mutex::new(Blockchain::new()))
        } else {
            info!(blocks = blocks.len(), "loaded blocks from storage");
            let mut bc = Blockchain::new();
            // Skip genesis (index 0) since Blockchain::new() creates it
            for block in blocks.into_iter().filter(|b| b.index > 0) {
                if let Err(e) = bc.validate_and_add_block(block) {
                    warn!(error = %e, "failed to load block from storage");
                }
            }
            Arc::new(Mutex::new(bc))
        }
    };

    {
        let bc = blockchain.lock().await;
        info!(height = bc.height(), "chain initialized");
    }

    // Load or create state
    let state = {
        let loaded_state = storage.load_state()?;
        if loaded_state.accounts.is_empty() {
            info!("creating new state with genesis balances");
            let mut state = State::with_genesis(vec![("genesis_account".to_string(), 1_000_000)]);
            state.chain_params.block_reward = config.producer.block_reward;
            state.chain_params.block_interval_secs = config.producer.block_interval_secs;
            state.chain_params.max_txs_per_block = config.producer.max_txs_per_block;
            state.chain_params.stake_weight = config.producer.stake_weight.clamp(0.0, 1.0);
            state.slashing_config = config.slashing.clone();
            Arc::new(Mutex::new(state))
        } else {
            info!(
                accounts = loaded_state.accounts.len(),
                "loaded state from storage"
            );
            Arc::new(Mutex::new(loaded_state))
        }
    };

    // Initialize mempool
    let mempool = Arc::new(Mutex::new(Mempool::new()));
    info!("mempool initialized");
    // Start P2P networking
    let port = config.node.p2p_port;

    let (mut p2p_service, p2p_handle, p2p_command_rx) = P2PService::new(port).await?;

    // Get shared state for monitoring (lock-free peer count access)
    let p2p_shared_state = p2p_handle.shared_state();

    // Get local peer ID for producer config
    let local_peer_id = p2p_handle.local_peer_id().to_string();
    info!(peer_id = %local_peer_id, "local peer id initialized");

    // Clone for various tasks
    let _local_id_metrics = local_peer_id.clone();
    let local_id_challenge = local_peer_id.clone();

    // Initialize block producer with PoI consensus
    let producer_config = ProducerConfig {
        max_txs_per_block: config.producer.max_txs_per_block,
        block_interval_secs: config.producer.block_interval_secs,
        node_id: local_peer_id.clone(),
        block_reward: config.producer.block_reward,
    };
    let producer = Arc::new(Mutex::new(BlockProducer::with_stake_weight(
        producer_config,
        config.producer.stake_weight,
    )));

    {
        let mut producer_guard = producer.lock().await;
        producer_guard.set_min_attestations(config.aggregator.min_attestations);
    }

    // Register self as validator with default metrics
    {
        let mut producer_guard = producer.lock().await;
        let metrics = BlockProducer::default_node_metrics(local_peer_id.clone());
        producer_guard.register_self(metrics);
        info!(score = ?producer_guard.get_my_score(), "registered local validator");
    }

    // ===== Initialize Proof of Internet (PoI) Services =====

    // Measurement service for real internet speed testing
    let measurement_config = config.measurement.clone();
    let measurement_service = Arc::new(MeasurementService::new(measurement_config));
    info!("measurement service initialized");

    // Metrics aggregator for peer attestations and reputation
    let aggregator_config = config.aggregator.clone();
    let metrics_aggregator = Arc::new(Mutex::new(MetricsAggregator::new(aggregator_config)));

    // Register self in aggregator
    {
        let mut agg = metrics_aggregator.lock().await;
        agg.register_node(local_peer_id.clone());
    }
    sync_validator_metrics_from_aggregator(&producer, &metrics_aggregator, &state, &local_peer_id)
        .await;
    info!("metrics aggregator initialized");

    // Epoch manager for validator rotation and rewards
    let epoch_config = EpochConfig {
        blocks_per_epoch: config.epoch.blocks_per_epoch,
        history_epochs: config.epoch.history_epochs,
        min_attestations_for_validator: config.epoch.min_attestations_for_validator,
        max_active_validators: config.epoch.max_active_validators,
        reputation_decay: config.epoch.reputation_decay,
        missed_epoch_slash_bps: config.epoch.missed_epoch_slash_bps,
        top_performer_bonus_bps: config.epoch.top_performer_bonus_bps,
        top_performer_count: config.epoch.top_performer_count,
    };
    let epoch_manager = Arc::new(EpochManager::new(
        epoch_config,
        vec![local_peer_id.clone()],
    ));
    info!("epoch manager initialized");

    // Metric challenge service for P2P bandwidth verification
    let challenge_config = config.metric_challenge.clone();
    let challenge_signing_key = ed25519_dalek::SigningKey::generate(&mut rand::rngs::OsRng);
    let metric_challenge = Arc::new(MetricChallengeService::new(
        challenge_config,
        challenge_signing_key,
        local_peer_id.clone(),
    ));
    info!("metric challenge service initialized");

    // Anti-gaming service for validation and rate limiting
    let anti_gaming_config = config.anti_gaming.clone();
    let anti_gaming = Arc::new(Mutex::new(AntiGamingService::new(anti_gaming_config)));
    info!("anti-gaming protections enabled");

    // Channel: P2P → main
    let (tx, mut rx) = tokio::sync::mpsc::channel(100);

    // Start P2P networking task - it now owns the service
    tokio::spawn(async move {
        p2p_service.run(tx, p2p_command_rx).await;
    });

    info!(p2p_port = port, "node ready and waiting for p2p events");

    // ===== Initialize DHT for peer discovery =====
    let dht_config = DhtConfig {
        bootstrap_nodes: config.dht.bootstrap_nodes.clone(),
        dht_port: config.dht.dht_port,
        enabled: config.dht.enabled,
        bootstrap_interval_secs: config.dht.bootstrap_interval_secs,
        enable_mdns: config.dht.enable_mdns,
        protocol_name: config.dht.protocol_name.clone(),
    };

    let (dht_event_tx, mut dht_event_rx) = mpsc::channel(100);

    let dht_handle = if dht_config.enabled {
        // Generate identity for DHT (could reuse p2p key in production)
        let dht_key = libp2p::identity::Keypair::generate_ed25519();

        match DhtService::new(dht_config.clone(), &dht_key, dht_event_tx).await {
            Ok((mut dht_service, dht_command_tx)) => {
                info!(
                    peer_id = %dht_service.local_peer_id(),
                    port = dht_config.dht_port,
                    "DHT service initialized"
                );

                // Register DHT peer address in metric challenge service
                let dht_addr = std::net::SocketAddr::new(
                    std::net::IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)),
                    dht_config.dht_port,
                );
                metric_challenge
                    .register_peer_address(local_peer_id.clone(), dht_addr)
                    .await;

                // Spawn DHT task
                tokio::spawn(async move {
                    dht_service.run().await;
                });

                Some(dht_command_tx)
            }
            Err(e) => {
                warn!(error = %e, "failed to initialize DHT service");
                None
            }
        }
    } else {
        info!("DHT peer discovery disabled");
        None
    };

    // Start RPC server for wallet CLI
    let rpc_state = Arc::new(RpcState {
        blockchain: blockchain.clone(),
        state: state.clone(),
        mempool: mempool.clone(),
        p2p: p2p_handle.clone(),
    });

    let rpc_bind_addr = config.rpc.bind_addr.clone();
    let rpc_port = config.rpc.port;
    tokio::spawn(async move {
        if let Err(e) = start_rpc_server(rpc_state, &rpc_bind_addr, rpc_port).await {
            error!(error = %e, "rpc server error");
        }
    });

    if config.monitoring.enabled {
        let monitoring_state = Arc::new(MonitoringState {
            blockchain: blockchain.clone(),
            mempool: mempool.clone(),
            p2p_shared: p2p_shared_state.clone(),
            producer: producer.clone(),
            aggregator: metrics_aggregator.clone(),
            measurement: measurement_service.clone(),
            state: state.clone(),
            started_at: Instant::now(),
        });
        let monitoring_bind_addr = config.monitoring.bind_addr.clone();
        let monitoring_port = config.monitoring.port;

        tokio::spawn(async move {
            if let Err(e) =
                start_monitoring_server(monitoring_state, &monitoring_bind_addr, monitoring_port)
                    .await
            {
                error!(error = %e, "monitoring server error");
            }
        });
    }

    // ===== WebSocket Event Channel =====
    let ws_event_tx = websocket::create_event_channel();

    if config.websocket.enabled {
        let ws_bind_addr = config.websocket.bind_addr.clone();
        let ws_port = config.websocket.port;
        let ws_tx = ws_event_tx.clone();

        tokio::spawn(async move {
            if let Err(e) = start_ws_server(&ws_bind_addr, ws_port, ws_tx).await {
                error!(error = %e, "websocket server error");
            }
        });
    }

    // ===== Periodic Metric Measurement Task =====
    // Measures own internet performance periodically and announces to network
    let measurement_task = measurement_service.clone();
    let aggregator_task = metrics_aggregator.clone();
    let producer_task_metrics = producer.clone();
    let state_task_metrics = state.clone();
    let p2p_metrics = p2p_handle.clone();
    let local_id_metrics = local_peer_id.clone();
    let measurement_interval_secs = config.producer.metric_measurement_interval_secs;

    tokio::spawn(async move {
        let mut measurement_interval = interval(Duration::from_secs(measurement_interval_secs));

        loop {
            measurement_interval.tick().await;

            // Run measurement cycle
            let metrics = measurement_task.run_measurement_cycle().await;

            // Update self-reported metrics in aggregator
            {
                let mut agg = aggregator_task.lock().await;
                agg.update_self_reported(
                    &local_id_metrics,
                    metrics.download_mbps,
                    metrics.upload_mbps,
                    metrics.latency_ms,
                    metrics.uptime_percent,
                    metrics.stability_percent,
                );
            }

            sync_validator_metrics_from_aggregator(
                &producer_task_metrics,
                &aggregator_task,
                &state_task_metrics,
                &local_id_metrics,
            )
            .await;

            let attestation_count = {
                let agg = aggregator_task.lock().await;
                agg.get_consensus_node_metrics(&local_id_metrics)
                    .map(|metrics| metrics.attestation_count)
                    .unwrap_or(0)
            };

            // Announce metrics to network
            p2p_metrics.announce_metrics(
                metrics.download_mbps,
                metrics.upload_mbps,
                metrics.latency_ms,
                metrics.uptime_percent,
                metrics.stability_percent,
                attestation_count,
            );

            if metrics.sample_count > 0 {
                info!(
                    download_mbps = metrics.download_mbps,
                    upload_mbps = metrics.upload_mbps,
                    latency_ms = metrics.latency_ms,
                    "metrics updated"
                );
            }
        }
    });

    // Block production task (runs every block_interval_secs)
    let p2p_producer = p2p_handle.clone();
    let blockchain_producer = blockchain.clone();
    let mempool_producer = mempool.clone();
    let state_producer = state.clone();
    let producer_task = producer.clone();
    let storage_producer = storage.clone();
    let aggregator_epoch = metrics_aggregator.clone();
    let ws_tx_producer = ws_event_tx.clone();
    let mempool_ttl_secs = config.producer.mempool_ttl_secs;

    tokio::spawn(async move {
        loop {
            let block_interval_secs = {
                let state_guard = state_producer.lock().await;
                state_guard.chain_params.block_interval_secs.max(1)
            };
            sleep(Duration::from_secs(block_interval_secs)).await;

            // Expire stale mempool transactions before producing a block.
            {
                let mut mempool_guard = mempool_producer.lock().await;
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                let expired = mempool_guard.expire_old(now, mempool_ttl_secs);
                if expired > 0 {
                    info!(expired, "expired stale mempool transactions");
                }
            }

            let mut producer_guard = producer_task.lock().await;

            if let Some((new_block, txs, executed_proposals)) = producer_guard
                .produce_block(
                    &blockchain_producer,
                    &mempool_producer,
                    &state_producer,
                    &ws_tx_producer,
                )
                .await
            {
                let block_height = new_block.index;
                info!(
                    block_height,
                    tx_count = txs.len(),
                    proposals_executed = executed_proposals.len(),
                    "produced block"
                );

                // Save block to storage
                if let Err(e) = storage_producer.save_block(&new_block) {
                    error!(error = %e, "failed to save produced block");
                }

                // Save state to storage
                {
                    let state_guard = state_producer.lock().await;
                    if let Err(e) = storage_producer.save_state(&state_guard) {
                        error!(error = %e, "failed to save state after block production");
                    }
                }

                // Remove included transactions from mempool
                if !txs.is_empty() {
                    let mut mempool_guard = mempool_producer.lock().await;
                    mempool_guard.remove_transactions(&txs);
                }

                // Broadcast block
                let json = serde_json::to_string(&new_block).unwrap();
                p2p_producer.publish_block(json);
                info!(block_height, "broadcasted block");

                // Broadcast new block to WebSocket subscribers
                let _ = ws_tx_producer.send(WsEvent::NewBlock {
                    index: new_block.index,
                    hash: new_block.hash.clone(),
                    validator: new_block.validator.clone(),
                    tx_count: txs.len(),
                    timestamp: new_block.timestamp.to_rfc3339(),
                });

                // Record block production in epoch manager
                epoch_manager
                    .record_block_production(
                        &local_peer_id,
                        block_height,
                        txs.len(),
                        (block_interval_secs as f64) * 1000.0,
                    )
                    .await;

                // Check for epoch boundary
                {
                    let mut agg = aggregator_epoch.lock().await;
                    if agg.should_start_new_epoch(block_height) {
                        let snapshot = agg.end_epoch(block_height);
                        info!(
                            epoch = snapshot.epoch_number,
                            end_block = snapshot.end_block,
                            nodes = snapshot.node_scores.len(),
                            "metrics epoch ended"
                        );
                    }
                }
            }
        }
    });

    // ===== Automatic Metric Challenge Task =====
    // Periodically challenges unverified peers to verify their metrics
    let challenge_service = metric_challenge.clone();
    let challenge_p2p = p2p_handle.clone();
    let challenge_aggregator = metrics_aggregator.clone();
    let challenge_config_clone = config.metric_challenge.clone();
    let challenge_local_id = local_id_challenge;

    if challenge_config_clone.auto_challenge_enabled {
        tokio::spawn(async move {
            let mut challenge_interval = interval(Duration::from_secs(60));

            loop {
                challenge_interval.tick().await;

                // Get unverified nodes that need challenges
                let unverified = {
                    let agg = challenge_aggregator.lock().await;
                    agg.get_unverified_nodes()
                };

                for node_id in unverified {
                    if node_id == challenge_local_id {
                        continue; // Don't challenge ourselves
                    }

                    if challenge_service.can_challenge_peer(&node_id).await {
                        if let Some(challenge) = challenge_service.create_challenge(node_id.clone()).await {
                            info!(
                                target = challenge.target_id,
                                nonce = challenge.challenge_nonce,
                                "initiating metric challenge"
                            );
                            
                            // Send challenge via P2P
                            challenge_p2p.send_metric_challenge(
                                challenge.target_id.clone(),
                                challenge.challenge_nonce.clone(),
                                challenge.bytes_to_download,
                            );
                        }
                    }
                }
                
                // Cleanup expired challenges
                let cleaned = challenge_service.cleanup_expired_challenges().await;
                if cleaned > 0 {
                    debug!(cleaned, "cleaned up expired challenges");
                }
            }
        });
    }

    // ===== DHT Event Handler =====
    // Handle peer discovery events from DHT
    if let Some(_dht_command_tx) = dht_handle {
        let _dht_metric_challenge = metric_challenge.clone();
        let dht_aggregator = metrics_aggregator.clone();
        let _dht_producer = producer.clone();
        let _dht_state = state.clone();
        let dht_p2p = p2p_handle.clone();
        
        tokio::spawn(async move {
            while let Some(dht_event) = dht_event_rx.recv().await {
                match dht_event {
                    DhtEvent::PeerDiscovered(peer_id) => {
                        info!(peer_id = %peer_id, "DHT discovered new peer");
                        // Register peer in aggregator for potential attestation
                        {
                            let mut agg = dht_aggregator.lock().await;
                            agg.register_node(peer_id.to_string());
                        }
                    }
                    DhtEvent::PeerConnected(peer_id) => {
                        info!(peer_id = %peer_id, "DHT connected to peer");
                        // Request metric verification for new peer
                        let nonce = format!("{:x}", rand::random::<u64>());
                        dht_p2p.send_metric_challenge(
                            peer_id.to_string(),
                            nonce,
                            1_000_000, // 1 MB initial challenge
                        );
                    }
                    DhtEvent::PeerDisconnected(peer_id) => {
                        info!(peer_id = %peer_id, "DHT peer disconnected");
                    }
                    DhtEvent::BootstrapComplete => {
                        info!("DHT bootstrap completed");
                    }
                    DhtEvent::RecordFound { key, value } => {
                        debug!(key_len = key.len(), value_len = value.len(), "DHT record found");
                    }
                }
            }
        });
    }

    // Main event loop
    let storage_main = storage.clone();
    let ws_tx_main = ws_event_tx.clone();
    let p2p = p2p_handle.clone();
    while let Some(event) = rx.recv().await {
        match event {
            P2PEvent::Message(NetworkMessage::Block(block_json)) => {
                info!("received block data");

                match serde_json::from_str::<Block>(&block_json) {
                    Ok(block) => {
                        let mut bc = blockchain.lock().await;
                        let block_index = block.index;
                        let tx_count = block.transactions.len();

                        // Basic block validity (header, hash, merkle root, tx signatures) against our tip.
                        if let Err(e) = bc.validate_next_block(&block) {
                            warn!(error = %e, "rejected block");
                            continue;
                        }

                        // Verify that the block's validator matches the expected PoI-selected validator.
                        {
                            let producer_guard = producer.lock().await;
                            let state_guard = state.lock().await;
                            let stakes = state_guard.get_stake_map();
                            if let Some(expected_validator) = producer_guard.select_validator(
                                &bc.last_block().hash,
                                block.index,
                                &stakes,
                            ) {
                                if block.validator != expected_validator {
                                    warn!(
                                        block_validator = %block.validator,
                                        expected_validator = %expected_validator,
                                        block_index = block.index,
                                        "rejected block: validator mismatch"
                                    );
                                    continue;
                                }
                            }
                            // If select_validator returns None (no validators registered),
                            // skip this check — single-node bootstrapping scenario.
                        }

                        let block_time_secs: u64 = match block.timestamp.timestamp().try_into() {
                            Ok(v) => v,
                            Err(_) => {
                                warn!("rejected block: timestamp before unix epoch");
                                continue;
                            }
                        };

                        // Validate and apply state transition atomically. Reject the block if any tx fails.
                        {
                            let mut state_guard = state.lock().await;

                            // Enforce runtime chain parameters.
                            if tx_count > state_guard.chain_params.max_txs_per_block {
                                warn!(
                                    tx_count,
                                    max_txs = state_guard.chain_params.max_txs_per_block,
                                    "rejected block: too many transactions"
                                );
                                continue;
                            }

                            let mut next_state = state_guard.clone();
                            let mut tx_error = None;
                            let mut tx_events = Vec::new();
                            for tx in &block.transactions {
                                match next_state.apply_transaction_at(tx, block_time_secs) {
                                    Ok(maybe_event) => {
                                        if let Some(event) = maybe_event {
                                            tx_events.push(event);
                                        }
                                    }
                                    Err(e) => {
                                        tx_error = Some(e);
                                        break;
                                    }
                                }
                            }

                            if let Some(e) = tx_error {
                                warn!(error = ?e, "rejected block: invalid transaction");

                                // Slash the validator for proposing an invalid block.
                                let slash_reason = SlashReason::InvalidBlockProposal;
                                drop(state_guard);
                                slash_validator(
                                    &state,
                                    &producer,
                                    &storage_main,
                                    &ws_tx_main,
                                    &block.validator,
                                    slash_reason,
                                )
                                .await;
                                continue;
                            }

                            // Credit validator with fees + block reward (from current chain params).
                            let block_reward = next_state.chain_params.block_reward;
                            next_state.apply_block_rewards(
                                &block.validator,
                                &block.transactions,
                                block_reward,
                            );

                            // Execute governance actions deterministically at block time.
                            let executed_proposals =
                                next_state.execute_passed_proposals_at(block_time_secs);

                            *state_guard = next_state;

                            for executed in executed_proposals {
                                let _ = ws_tx_main.send(WsEvent::ProposalUpdate {
                                    proposal_id: executed.proposal_id,
                                    title: executed.title,
                                    status: "Passed".to_string(),
                                    yes_votes: executed.yes_votes,
                                    no_votes: executed.no_votes,
                                });
                            }
                        }

                        // Commit block to chain after state transition succeeds.
                        bc.chain.push(block.clone());
                        info!(
                            block_index,
                            tx_count,
                            chain_height = bc.height(),
                            "accepted block"
                        );

                        // Remove included transactions from mempool
                        if !block.transactions.is_empty() {
                            let mut mempool_guard = mempool.lock().await;
                            mempool_guard.remove_transactions(&block.transactions);
                        }

                        // Save block and state to storage
                        if let Err(e) = storage_main.save_block(&block) {
                            warn!(error = %e, "failed to persist accepted block");
                        }
                        {
                            let state_guard = state.lock().await;
                            if let Err(e) = storage_main.save_state(&state_guard) {
                                warn!(error = %e, "failed to persist state after accepted block");
                            }
                        }

                        // Broadcast new block event to WebSocket subscribers
                        let _ = ws_tx_main.send(WsEvent::NewBlock {
                            index: block_index,
                            hash: block.hash.clone(),
                            validator: block.validator.clone(),
                            tx_count,
                            timestamp: block.timestamp.to_rfc3339(),
                        });
                    }
                    Err(e) => {
                        warn!(error = %e, "failed to deserialize block");
                    }
                }
            }

            P2PEvent::Message(NetworkMessage::Transaction(tx_json)) => {
                info!("received transaction");

                match serde_json::from_str::<SignedTransaction>(&tx_json) {
                    Ok(signed_tx) => {
                        let state_guard = state.lock().await;
                        let mut mempool_guard = mempool.lock().await;

                        match mempool_guard.add_transaction(signed_tx.clone(), &state_guard) {
                            Ok(_) => {
                                info!(
                                    mempool_size = mempool_guard.len(),
                                    "transaction added to mempool"
                                );

                                // Broadcast new transaction event to WebSocket subscribers
                                let tx_type_str = format!("{:?}", signed_tx.tx.tx_type);
                                let _ = ws_tx_main.send(WsEvent::NewTransaction {
                                    tx_hash: signed_tx.tx_hash_hex(),
                                    sender: signed_tx.tx.sender.clone(),
                                    receiver: signed_tx.tx.receiver.clone(),
                                    amount: signed_tx.tx.amount,
                                    tx_type: tx_type_str,
                                });
                            }
                            Err(e) => {
                                warn!(error = ?e, "transaction rejected");
                            }
                        }
                    }
                    Err(e) => {
                        warn!(error = %e, "failed to deserialize transaction");
                    }
                }
            }

            P2PEvent::Message(NetworkMessage::ChainSyncRequest { from_height }) => {
                info!(from_height, "received chain sync request");

                let bc = blockchain.lock().await;
                let blocks: Vec<String> = bc
                    .get_blocks_from(from_height)
                    .iter()
                    .filter_map(|b| serde_json::to_string(b).ok())
                    .collect();

                if !blocks.is_empty() {
                    p2p.send_chain_sync_response(blocks.clone());
                    info!(blocks = blocks.len(), "sent chain sync response");
                }
            }

            P2PEvent::Message(NetworkMessage::ChainSyncResponse { blocks }) => {
                info!(blocks = blocks.len(), "received chain sync response");

                let parsed_blocks = match parse_chain_sync_blocks(&blocks) {
                    Ok(parsed) => parsed,
                    Err(e) => {
                        warn!(error = %e, "rejected chain sync response");
                        continue;
                    }
                };

                if !parsed_blocks.is_empty() {
                    let mut bc = blockchain.lock().await;
                    let old_chain = bc.chain.clone();
                    let old_height = bc.height();

                    match bc.sync_from_blocks(parsed_blocks) {
                        Ok(result) => {
                            if result.added == 0 {
                                continue;
                            }

                            let new_chain = bc.chain.clone();
                            match rebuild_state_from_chain(&new_chain) {
                                Ok(rebuilt_state) => {
                                    {
                                        let mut state_guard = state.lock().await;
                                        *state_guard = rebuilt_state;
                                    }

                                    // Remove included transactions from mempool (best-effort).
                                    {
                                        let mut mempool_guard = mempool.lock().await;
                                        if result.reorged {
                                            for block in &new_chain {
                                                mempool_guard
                                                    .remove_transactions(&block.transactions);
                                            }
                                        } else {
                                            for block in
                                                new_chain.iter().filter(|b| b.index > old_height)
                                            {
                                                mempool_guard
                                                    .remove_transactions(&block.transactions);
                                            }
                                        }
                                    }

                                    // Persist blocks + rebuilt state.
                                    if result.reorged {
                                        if let Err(e) = storage_main.clear() {
                                            warn!(
                                                error = %e,
                                                "failed to clear storage after reorg"
                                            );
                                        }
                                        for block in &new_chain {
                                            if let Err(e) = storage_main.save_block(block) {
                                                warn!(
                                                    error = %e,
                                                    index = block.index,
                                                    "failed to persist synced block"
                                                );
                                            }
                                        }
                                    } else {
                                        for block in
                                            new_chain.iter().filter(|b| b.index > old_height)
                                        {
                                            if let Err(e) = storage_main.save_block(block) {
                                                warn!(
                                                    error = %e,
                                                    index = block.index,
                                                    "failed to persist synced block"
                                                );
                                            }
                                        }
                                    }
                                    if let Err(e) = {
                                        let state_guard = state.lock().await;
                                        storage_main.save_state(&state_guard)
                                    } {
                                        warn!(
                                            error = %e,
                                            "failed to persist rebuilt state after sync"
                                        );
                                    }

                                    if result.reorged {
                                        info!(
                                            added = result.added,
                                            new_height = bc.height(),
                                            "chain reorganized during sync"
                                        );
                                    } else {
                                        info!(
                                            added = result.added,
                                            new_height = bc.height(),
                                            "synced new blocks"
                                        );
                                    }
                                }
                                Err(e) => {
                                    // Reject sync if state transition is invalid.
                                    bc.chain = old_chain;
                                    warn!(
                                        error = %e,
                                        old_height,
                                        "rejected synced chain: invalid state transition"
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            warn!(error = %e, "chain sync failed");
                        }
                    }
                }
            }

            P2PEvent::PeerConnected(peer) => {
                info!(peer = %peer, "peer connected");

                // Register peer as potential validator with default metrics
                {
                    let mut producer_guard = producer.lock().await;
                    let metrics = BlockProducer::default_node_metrics(peer.to_string());
                    producer_guard.register_peer(peer.to_string(), metrics);
                    info!(
                        validators = producer_guard.validator_count(),
                        "validator pool updated after peer connect"
                    );
                }

                // Register peer in metrics aggregator
                {
                    let mut agg = metrics_aggregator.lock().await;
                    agg.register_node(peer.to_string());
                }

                sync_validator_metrics_from_aggregator(
                    &producer,
                    &metrics_aggregator,
                    &state,
                    &peer.to_string(),
                )
                .await;

                // Request chain sync from new peer
                {
                    let height = {
                        let bc = blockchain.lock().await;
                        bc.height()
                    };
                    p2p.request_chain_sync(height + 1);
                    info!(from_height = height + 1, "requested chain sync from peer");

                    // Issue a metric challenge to the new peer
                    let nonce = format!("{:x}", rand::random::<u64>());
                    p2p.send_metric_challenge(
                        peer.to_string(),
                        nonce,
                        1_000_000, // 1 MB challenge
                    );
                    info!(peer = %peer, "sent metric challenge to peer");
                }
            }

            P2PEvent::PeerDisconnected(peer) => {
                info!(peer = %peer, "peer disconnected");

                // Remove peer from validator pool
                {
                    let mut producer_guard = producer.lock().await;
                    producer_guard.remove_peer(&peer.to_string());
                    info!(
                        validators = producer_guard.validator_count(),
                        "validator pool updated after peer disconnect"
                    );
                }

                // Note: We don't remove from aggregator - keep historical metrics
                // This allows reputation to persist across reconnections
            }

            // ===== Proof of Internet Metric Messages =====
            P2PEvent::Message(NetworkMessage::MetricChallenge {
                challenger_id,
                target_id,
                challenge_nonce,
                bytes_to_download,
                timestamp: _,
            }) => {
                let local_id = p2p.local_peer_id().to_string();

                // Only respond if we're the target
                if target_id == local_id {
                    info!(
                        challenger_id,
                        challenge_nonce, bytes_to_download, "received metric challenge"
                    );

                    // Check rate limiting
                    let should_respond = {
                        let mut ag = anti_gaming.lock().await;
                        ag.can_receive_challenge(&challenger_id).is_valid()
                    };

                    if !should_respond {
                        warn!(challenger_id, "rate limited metric challenge");
                    } else {
                        // Record that we received a challenge
                        {
                            let mut ag = anti_gaming.lock().await;
                            ag.record_challenge_received(&challenger_id);
                        }
                        // Perform actual measurement using our measurement service
                        let metrics = measurement_service.get_metrics().await;

                        // If we have real measurements, use them; otherwise run a quick cycle
                        let (download, upload, latency) = if metrics.sample_count > 0 {
                            (
                                metrics.download_mbps,
                                metrics.upload_mbps,
                                metrics.latency_ms,
                            )
                        } else {
                            // Run a measurement cycle to get fresh data
                            let fresh = measurement_service.run_measurement_cycle().await;
                            (fresh.download_mbps, fresh.upload_mbps, fresh.latency_ms)
                        };

                        // Calculate approximate duration based on bytes and speed
                        let duration_ms = if download > 0.0 {
                            ((bytes_to_download as f64 * 8.0) / (download * 1_000_000.0) * 1000.0)
                                as u64
                        } else {
                            1000
                        };

                        p2p.send_metric_response(
                            challenge_nonce,
                            download,
                            upload,
                            latency,
                            bytes_to_download,
                            duration_ms,
                        );
                        info!(
                            download,
                            upload, latency, duration_ms, "responded to metric challenge"
                        );
                    }
                }
            }

            P2PEvent::Message(NetworkMessage::MetricChallengeResponse {
                challenge_nonce,
                responder_id,
                download_mbps,
                upload_mbps,
                latency_ms,
                bytes_transferred,
                duration_ms,
                timestamp,
            }) => {
                info!(
                    responder_id,
                    challenge_nonce,
                    download_mbps,
                    upload_mbps,
                    latency_ms,
                    bytes_transferred,
                    duration_ms,
                    "received metric challenge response"
                );

                let local_id = p2p.local_peer_id().to_string();

                // Validate metrics with anti-gaming
                let validation = {
                    let mut ag = anti_gaming.lock().await;
                    // For response validation, use 0 attestations since this is raw measurement
                    ag.validate_metrics(&responder_id, download_mbps, upload_mbps, latency_ms, 0)
                };

                if !validation.is_valid() {
                    warn!(validation = ?validation, responder_id, "metric challenge response validation failed");
                } else {
                    // Verify the response makes sense (bytes/time roughly matches claimed speed)
                    let claimed_speed = if duration_ms > 0 {
                        (bytes_transferred as f64 * 8.0)
                            / (duration_ms as f64 / 1000.0)
                            / 1_000_000.0
                    } else {
                        0.0
                    };

                    // Allow 50% variance between claimed and calculated
                    let speed_ratio = if download_mbps > 0.0 {
                        claimed_speed / download_mbps
                    } else {
                        0.0
                    };

                    let confidence = if speed_ratio > 0.5 && speed_ratio < 2.0 {
                        0.8 // Good confidence if speeds roughly match
                    } else {
                        0.3 // Low confidence if mismatch
                    };

                    // Create and store attestation
                    let attestation = Attestation {
                        attester_id: local_id.clone(),
                        subject_id: responder_id.clone(),
                        download_mbps,
                        upload_mbps,
                        latency_ms,
                        confidence,
                        timestamp,
                        signature: String::new(), // TODO: Add proper signing
                    };

                    // Store in aggregator
                    {
                        let mut agg = metrics_aggregator.lock().await;
                        match agg.add_attestation(attestation) {
                            Ok(()) => info!(
                                responder_id,
                                confidence, "created attestation from metric challenge response"
                            ),
                            Err(e) => {
                                warn!(error = %e, responder_id, "failed to store attestation")
                            }
                        }
                    }

                    sync_validator_metrics_from_aggregator(
                        &producer,
                        &metrics_aggregator,
                        &state,
                        &responder_id,
                    )
                    .await;

                    // Broadcast attestation to network
                    p2p.send_metric_attestation(
                        responder_id,
                        download_mbps,
                        upload_mbps,
                        latency_ms,
                        confidence,
                        "".to_string(), // Signature placeholder - would need proper signing
                    );
                }
            }

            P2PEvent::Message(NetworkMessage::MetricAttestation {
                attester_id,
                subject_id,
                download_mbps,
                upload_mbps,
                latency_ms,
                confidence,
                timestamp,
                signature: _,
            }) => {
                let local_id = p2p.local_peer_id().to_string();

                // Don't process our own attestations
                if attester_id == local_id {
                    continue;
                }

                info!(
                    attester_id,
                    subject_id,
                    download_mbps,
                    upload_mbps,
                    latency_ms,
                    confidence,
                    "received metric attestation"
                );

                // Validate the attested values aren't obviously fake
                let validation = {
                    let ag = anti_gaming.lock().await;
                    ag.validate_bounds(download_mbps, upload_mbps, latency_ms)
                };

                if !validation.is_valid() {
                    warn!(validation = ?validation, subject_id, "rejected invalid attestation metrics");
                } else {
                    // Store the attestation
                    let attestation = Attestation {
                        attester_id: attester_id.clone(),
                        subject_id: subject_id.clone(),
                        download_mbps,
                        upload_mbps,
                        latency_ms,
                        confidence,
                        timestamp,
                        signature: String::new(), // TODO: Verify signature
                    };

                    let mut agg = metrics_aggregator.lock().await;
                    match agg.add_attestation(attestation) {
                        Ok(()) => info!(subject_id, "stored attestation"),
                        Err(e) => warn!(error = %e, subject_id, "failed to store attestation"),
                    }
                }

                sync_validator_metrics_from_aggregator(
                    &producer,
                    &metrics_aggregator,
                    &state,
                    &subject_id,
                )
                .await;
            }

            P2PEvent::Message(NetworkMessage::MetricAnnouncement {
                node_id,
                download_mbps,
                upload_mbps,
                latency_ms,
                uptime_percent,
                stability_percent,
                timestamp: _,
                attestation_count,
            }) => {
                let local_id = p2p.local_peer_id().to_string();

                // Don't process our own announcements
                if node_id == local_id {
                    continue;
                }

                info!(
                    node_id,
                    download_mbps,
                    upload_mbps,
                    latency_ms,
                    uptime_percent,
                    stability_percent,
                    attestation_count,
                    "received metric announcement"
                );

                // Validate with anti-gaming
                let validation = {
                    let mut ag = anti_gaming.lock().await;
                    ag.validate_metrics(
                        &node_id,
                        download_mbps,
                        upload_mbps,
                        latency_ms,
                        attestation_count,
                    )
                };

                match validation {
                    anti_gaming::ValidationResult::Valid => {
                        // Update aggregator with self-reported metrics
                        {
                            let mut agg = metrics_aggregator.lock().await;
                            agg.update_self_reported(
                                &node_id,
                                download_mbps,
                                upload_mbps,
                                latency_ms,
                                uptime_percent,
                                stability_percent,
                            );
                        }

                        sync_validator_metrics_from_aggregator(
                            &producer,
                            &metrics_aggregator,
                            &state,
                            &node_id,
                        )
                        .await;
                        info!(node_id, "accepted and stored announced metrics");
                    }
                    anti_gaming::ValidationResult::InsufficientAttestations => {
                        info!(node_id, "insufficient attestations, issuing challenge");
                        // Issue a challenge to verify
                        let nonce = format!("{:x}", rand::random::<u64>());
                        p2p.send_metric_challenge(
                            node_id.clone(),
                            nonce,
                            1_000_000, // 1 MB
                        );
                    }
                    other => {
                        warn!(validation = ?other, node_id, "rejected announced metrics");

                        // Slash for metric fraud on Outlier, OutOfBounds, or Suspicious results.
                        // RateLimited is not a fraud signal — skip slashing for it.
                        let should_slash = matches!(
                            other,
                            anti_gaming::ValidationResult::Outlier(_)
                                | anti_gaming::ValidationResult::OutOfBounds(_)
                                | anti_gaming::ValidationResult::Suspicious(_)
                        );
                        if should_slash {
                            slash_validator(
                                &state,
                                &producer,
                                &storage_main,
                                &ws_tx_main,
                                &node_id,
                                SlashReason::MetricFraud,
                            )
                            .await;
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn rebuild_state_from_chain(chain: &[Block]) -> Result<State> {
    // NOTE: This must match the node's genesis state initialization.
    let mut state = State::with_genesis(vec![("genesis_account".to_string(), 1_000_000)]);

    for block in chain.iter().filter(|b| b.index > 0) {
        let block_time_secs: u64 = block.timestamp.timestamp().try_into().map_err(|_| {
            anyhow::anyhow!("block timestamp before unix epoch (index {})", block.index)
        })?;

        // Enforce runtime chain parameters at each block boundary.
        if block.transactions.len() > state.chain_params.max_txs_per_block {
            return Err(anyhow::anyhow!(
                "block {} has {} txs, exceeds max {}",
                block.index,
                block.transactions.len(),
                state.chain_params.max_txs_per_block
            ));
        }

        for tx in &block.transactions {
            let _ = state
                .apply_transaction_at(tx, block_time_secs)
                .map_err(|e| {
                    anyhow::anyhow!(
                        "block {} contains invalid transaction: {:?}",
                        block.index,
                        e
                    )
                })?;
        }

        let block_reward = state.chain_params.block_reward;
        state.apply_block_rewards(&block.validator, &block.transactions, block_reward);
        state.execute_passed_proposals_at(block_time_secs);
    }

    Ok(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use netchain::transaction::{
        generate_ed25519_keypair, pubkey_to_address_hex, ProposalAction, SignedTransaction,
        Transaction,
    };

    #[test]
    fn test_rebuild_state_applies_governance_actions() {
        let kp = generate_ed25519_keypair();
        let addr = pubkey_to_address_hex(&kp.verifying_key());

        let base_time = Utc.timestamp_opt(1_700_000_000, 0).single().unwrap();
        let genesis = Block::new_at(0, vec![], "0".to_string(), "genesis".to_string(), base_time, String::new());

        let reward_block_1 = Block::new_at(
            1,
            vec![],
            genesis.hash.clone(),
            addr.clone(),
            base_time + chrono::Duration::seconds(1),
            String::new(),
        );
        let reward_block_2 = Block::new_at(
            2,
            vec![],
            reward_block_1.hash.clone(),
            addr.clone(),
            base_time + chrono::Duration::seconds(2),
            String::new(),
        );
        let reward_block_3 = Block::new_at(
            3,
            vec![],
            reward_block_2.hash.clone(),
            addr.clone(),
            base_time + chrono::Duration::seconds(3),
            String::new(),
        );

        let stake_tx =
            SignedTransaction::sign_with_keypair(&Transaction::stake(addr.clone(), 100, 1, 0), &kp);
        let proposal_tx = SignedTransaction::sign_with_keypair(
            &Transaction::create_proposal_with_action(
                addr.clone(),
                1,
                1,
                "Change reward".to_string(),
                "Raise reward after restart".to_string(),
                2,
                ProposalAction::ChangeBlockReward(75),
            ),
            &kp,
        );
        let vote_tx = SignedTransaction::sign_with_keypair(
            &Transaction::vote_proposal(addr.clone(), 1, 2, 1, true),
            &kp,
        );

        let block1 = Block::new_at(
            4,
            vec![stake_tx, proposal_tx, vote_tx],
            reward_block_3.hash.clone(),
            addr.clone(),
            base_time + chrono::Duration::seconds(4),
            String::new(),
        );
        let block2 = Block::new_at(
            5,
            vec![],
            block1.hash.clone(),
            addr.clone(),
            base_time + chrono::Duration::seconds(7),
            String::new(),
        );

        let rebuilt = rebuild_state_from_chain(&[
            genesis,
            reward_block_1,
            reward_block_2,
            reward_block_3,
            block1,
            block2,
        ])
        .unwrap();
        assert_eq!(rebuilt.chain_params.block_reward, 75);
        assert!(rebuilt.get_proposal(1).is_none());
    }

    #[test]
    fn test_parse_chain_sync_blocks_is_strict() {
        let base_time = Utc.timestamp_opt(1_700_000_000, 0).single().unwrap();
        let block = Block::new_at(
            1,
            vec![],
            "genesis".to_string(),
            "validator".to_string(),
            base_time,
            String::new(),
        );
        let valid_json = serde_json::to_string(&block).unwrap();
        let invalid_json = "{not valid json}".to_string();

        let result = parse_chain_sync_blocks(&[valid_json, invalid_json]);
        assert!(result.is_err());
    }
}
