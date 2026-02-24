export enum EventType {
  PAYMENT = 'payment',
  OFFER = 'offer',
  CONTRACT = 'contract',
  ACCOUNT = 'account',
  TRUSTLINE = 'trustline',
  TRANSFER = 'transfer',
  MINT = 'mint',
  BURN = 'burn',
  STAKE = 'stake',
  UNSTAKE = 'unstake',
  REWARD = 'reward',
}

export enum DeliveryStatus {
  PENDING = 'pending',
  DELIVERED = 'delivered',
  FAILED = 'failed',
  RETRYING = 'retrying',
}

export enum ConsumerStatus {
  ACTIVE = 'active',
  INACTIVE = 'inactive',
  SUSPENDED = 'suspended',
}

export interface StellarEventData {
  id: string;
  eventType: EventType;
  ledgerSequence: number;
  timestamp: Date;
  transactionHash: string;
  sourceAccount: string;
  payload: Record<string, any>;
  createdAt: Date;
}

export interface PaymentEventPayload {
  amount: string;
  assetType: string;
  assetCode?: string;
  assetIssuer?: string;
  from: string;
  to: string;
  memo?: string;
}

export interface OfferEventPayload {
  offerId: string;
  seller: string;
  sellingAssetType: string;
  sellingAssetCode?: string;
  sellingAssetIssuer?: string;
  buyingAssetType: string;
  buyingAssetCode?: string;
  buyingAssetIssuer?: string;
  amount: string;
  price: string;
  type: 'create' | 'update' | 'delete';
}

export interface ContractEventPayload {
  contractId: string;
  topics: string[];
  data: any;
  function?: string;
  eventIndex: number;
}

export interface TransferEventPayload {
  from: string;
  to: string;
  amount: string;
  assetType: string;
  assetCode?: string;
  assetIssuer?: string;
  contractId?: string;
}

export interface MintEventPayload {
  to: string;
  amount: string;
  assetType: string;
  assetCode?: string;
  assetIssuer?: string;
  contractId: string;
}

export interface BurnEventPayload {
  from: string;
  amount: string;
  assetType: string;
  assetCode?: string;
  assetIssuer?: string;
  contractId: string;
}

export interface StakeEventPayload {
  user: string;
  amount: string;
  stakingContract: string;
  lockPeriod?: number;
  rewardMultiplier?: number;
}

export interface UnstakeEventPayload {
  user: string;
  amount: string;
  stakingContract: string;
  rewardsEarned?: string;
}

export interface RewardEventPayload {
  user: string;
  amount: string;
  rewardType: string;
  reason?: string;
  contractId: string;
}
