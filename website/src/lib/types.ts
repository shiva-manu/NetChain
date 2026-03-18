export interface NetChainEndpoints {
  rpcUrl: string;
  monitoringUrl: string;
  wsUrl: string;
}

export interface ChainInfo {
  height: number;
  latest_block_hash: string;
  genesis_hash: string;
}

export interface BlockSummary {
  index: number;
  hash: string;
  validator: string;
  tx_count: number;
  timestamp: string;
}

export interface ProposalActionChangeBlockReward {
  ChangeBlockReward: number;
}

export interface ProposalActionChangeBlockInterval {
  ChangeBlockInterval: number;
}

export interface ProposalActionChangeMaxTxsPerBlock {
  ChangeMaxTxsPerBlock: number;
}

export interface ProposalActionChangeStakeWeight {
  ChangeStakeWeight: number;
}

export type ProposalAction =
  | ProposalActionChangeBlockReward
  | ProposalActionChangeBlockInterval
  | ProposalActionChangeMaxTxsPerBlock
  | ProposalActionChangeStakeWeight;

export interface CreateProposalTxDetails {
  title: string;
  description: string;
  voting_period_secs: number;
  action?: ProposalAction | null;
}

export interface VoteProposalTxDetails {
  proposal_id: number;
  support: boolean;
}

export type TransactionType =
  | "Transfer"
  | "Stake"
  | "Unstake"
  | { CreateProposal: CreateProposalTxDetails }
  | { VoteProposal: VoteProposalTxDetails };

export interface TransactionRecord {
  sender: string;
  receiver: string;
  amount: number;
  fee: number;
  nonce: number;
  timestamp: number;
  tx_type: TransactionType;
  memo?: string | null;
}

export interface SignedTransaction {
  tx: TransactionRecord;
  signature: string;
  pubkey: string;
}

export interface BlockDetails {
  index: number;
  timestamp: string;
  merkle_root: string;
  transactions: SignedTransaction[];
  validator: string;
  previous_hash: string;
  hash: string;
}

export interface ProposalInfo {
  id: number;
  proposer?: string;
  title: string;
  description?: string;
  created_at?: number;
  expires_at?: number;
  yes_votes: number;
  no_votes: number;
  status: string;
  voter_count?: number;
}

export interface AccountInfo {
  address: string;
  balance: number;
  nonce: number;
  staked_balance: number;
}

export interface StakingInfo {
  address: string;
  staked_balance: number;
  total_staked: number;
}

export interface ChainParams {
  block_reward: number;
  block_interval_secs: number;
  max_txs_per_block: number;
  stake_weight: number;
  proposal_quorum_bps: number;
  proposal_approval_bps: number;
  min_proposal_stake: number;
}

export interface HealthSnapshot {
  status: string;
  consensus_mode: string;
  uptime_secs: number;
  chain_height: number;
  mempool_size: number;
  peer_count: number;
  validator_count: number;
  verified_validator_count: number;
  unverified_validator_count: number;
  aggregator_nodes: number;
  current_epoch: number;
  slashed_validator_count: number;
  average_reputation: number;
  average_identity_score: number;
}

export interface MetricsSnapshot {
  netchain_chain_height?: number;
  netchain_mempool_size?: number;
  netchain_peer_count?: number;
  netchain_validator_count?: number;
  netchain_verified_validator_count?: number;
  netchain_unverified_validator_count?: number;
  netchain_aggregator_nodes?: number;
  netchain_current_epoch?: number;
  netchain_slashed_validator_count?: number;
  netchain_average_reputation?: number;
  netchain_average_identity_score?: number;
  netchain_uptime_seconds?: number;
  netchain_download_mbps?: number;
  netchain_upload_mbps?: number;
  netchain_latency_ms?: number;
  netchain_uptime_percent?: number;
}

export interface TransactionFeedItem {
  tx_hash: string;
  sender: string;
  receiver: string;
  amount: number;
  tx_type: string;
}

export interface ValidatorSlashedEvent {
  validator: string;
  reason: string;
  amount_burned: number;
  remaining_stake: number;
}

export type WsEvent =
  | { event: "new_block"; data: BlockSummary }
  | { event: "new_transaction"; data: TransactionFeedItem }
  | { event: "proposal_update"; data: ProposalInfo }
  | { event: "validator_slashed"; data: ValidatorSlashedEvent };

export type WsTopic = "new_blocks" | "new_transactions" | "proposals" | "slashing";

export interface WalletWatchItem {
  label: string;
  address: string;
}

export interface DashboardSnapshot {
  health: HealthSnapshot | null;
  metrics: MetricsSnapshot | null;
  chainInfo: ChainInfo | null;
  chainParams: ChainParams | null;
}
