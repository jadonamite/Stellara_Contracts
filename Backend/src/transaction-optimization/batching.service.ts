import { Injectable } from '@nestjs/common';

export interface BatchTransaction {
  id: string;
  transactions: IndividualTransaction[];
  totalAmount: number;
  totalFees: number;
  status: 'pending' | 'processing' | 'completed' | 'failed';
  createdAt: Date;
  processedAt?: Date;
  savings?: BatchSavings;
}

export interface IndividualTransaction {
  id: string;
  recipient: string;
  amount: number;
  priority: TransactionPriority;
  fee: number;
  metadata?: any;
}

export interface BatchSavings {
  totalIndividualFees: number;
  batchFee: number;
  savings: number;
  savingsPercentage: number;
}

export interface BatchingConfig {
  maxBatchSize: number;
  batchTimeout: number; // in milliseconds
  minBatchSize: number;
  priorityThreshold: TransactionPriority;
  enableSmartBatching: boolean;
}

export interface BatchMetrics {
  totalBatches: number;
  totalTransactions: number;
  averageBatchSize: number;
  totalSavings: number;
  averageSavingsPercentage: number;
  processingTime: number;
}

export enum TransactionPriority {
  LOW = 1,
  MEDIUM = 2,
  HIGH = 3,
  CRITICAL = 4,
}

@Injectable()
export class BatchingService {
  private pendingTransactions: IndividualTransaction[] = [];
  private batchConfig: BatchingConfig;
  private batchMetrics: BatchMetrics;

  constructor() {
    this.batchConfig = {
      maxBatchSize: 100,
      batchTimeout: 5000, // 5 seconds
      minBatchSize: 5,
      priorityThreshold: TransactionPriority.HIGH,
      enableSmartBatching: true,
    };

    this.batchMetrics = {
      totalBatches: 0,
      totalTransactions: 0,
      averageBatchSize: 0,
      totalSavings: 0,
      averageSavingsPercentage: 0,
      processingTime: 0,
    };

    this.startBatchProcessor();
  }

  async addTransaction(transaction: IndividualTransaction): Promise<string> {
    const transactionId = this.generateTransactionId();
    const enrichedTransaction: IndividualTransaction = {
      ...transaction,
      id: transactionId,
    };

    this.pendingTransactions.push(enrichedTransaction);

    // Check if batch should be processed immediately
    if (this.shouldProcessImmediately(enrichedTransaction)) {
      await this.processBatch();
    }

    return transactionId;
  }

  async addTransactionsBatch(transactions: IndividualTransaction[]): Promise<string[]> {
    const transactionIds = transactions.map(tx => ({
      ...tx,
      id: this.generateTransactionId(),
    }));

    this.pendingTransactions.push(...transactionIds);

    if (this.shouldProcessImmediately()) {
      await this.processBatch();
    }

    return transactionIds.map(tx => tx.id);
  }

  async createOptimalBatch(transactions: IndividualTransaction[]): Promise<BatchTransaction> {
    // Sort transactions by priority and amount for optimal batching
    const sortedTransactions = this.sortTransactionsForBatching(transactions);
    
    // Group transactions by similar characteristics
    const batchGroups = this.groupTransactionsForBatching(sortedTransactions);
    
    // Calculate optimal batch composition
    const optimalBatch = this.optimizeBatchComposition(batchGroups);
    
    const batchId = this.generateBatchId();
    const totalAmount = optimalBatch.reduce((sum, tx) => sum + tx.amount, 0);
    const totalIndividualFees = optimalBatch.reduce((sum, tx) => sum + tx.fee, 0);
    const batchFee = this.calculateBatchFee(optimalBatch);
    const savings = this.calculateSavings(totalIndividualFees, batchFee);

    const batch: BatchTransaction = {
      id: batchId,
      transactions: optimalBatch,
      totalAmount,
      totalFees: batchFee,
      status: 'pending',
      createdAt: new Date(),
      savings,
    };

    this.updateMetrics(batch);
    return batch;
  }

  async processBatch(): Promise<BatchTransaction | null> {
    if (this.pendingTransactions.length < this.batchConfig.minBatchSize) {
      return null;
    }

    const transactionsToProcess = this.pendingTransactions.splice(0, this.batchConfig.maxBatchSize);
    const batch = await this.createOptimalBatch(transactionsToProcess);

    // Submit batch to blockchain (mock implementation)
    const processedBatch = await this.submitBatchToBlockchain(batch);

    // Update transaction statuses
    processedBatch.transactions.forEach(tx => {
      tx.status = processedBatch.status;
    });

    processedBatch.processedAt = new Date();
    processedBatch.status = processedBatch.status;

    return processedBatch;
  }

  async getBatchStatus(batchId: string): Promise<BatchTransaction | null> {
    // In a real implementation, query from database
    return null; // Mock implementation
  }

  async getBatchMetrics(period: 'hour' | 'day' | 'week' | 'month' = 'day'): Promise<BatchMetrics> {
    // Calculate metrics for the specified period
    return this.batchMetrics;
  }

  async updateBatchConfig(config: Partial<BatchingConfig>): Promise<BatchingConfig> {
    this.batchConfig = { ...this.batchConfig, ...config };
    
    // Validate configuration
    this.validateBatchConfig(this.batchConfig);
    
    return this.batchConfig;
  }

  private shouldProcessImmediately(transaction: IndividualTransaction): boolean {
    // Process immediately for high-priority transactions
    if (transaction.priority >= this.batchConfig.priorityThreshold) {
      return true;
    }

    // Process if batch is full
    if (this.pendingTransactions.length >= this.batchConfig.maxBatchSize) {
      return true;
    }

    return false;
  }

  private shouldProcessImmediately(): boolean {
    return this.pendingTransactions.length >= this.batchConfig.maxBatchSize;
  }

  private sortTransactionsForBatching(transactions: IndividualTransaction[]): IndividualTransaction[] {
    return transactions.sort((a, b) => {
      // First by priority (higher first)
      if (a.priority !== b.priority) {
        return b.priority - a.priority;
      }
      
      // Then by amount (smaller first for better batching)
      return a.amount - b.amount;
    });
  }

  private groupTransactionsForBatching(transactions: IndividualTransaction[]): IndividualTransaction[][] {
    const groups: IndividualTransaction[][] = [];
    
    // Group by recipient for batch optimization
    const recipientGroups = new Map<string, IndividualTransaction[]>();
    
    transactions.forEach(tx => {
      if (!recipientGroups.has(tx.recipient)) {
        recipientGroups.set(tx.recipient, []);
      }
      recipientGroups.get(tx.recipient)!.push(tx);
    });

    // Convert groups to array and sort by size
    recipientGroups.forEach(group => {
      groups.push(group);
    });

    return groups.sort((a, b) => b.length - a.length);
  }

  private optimizeBatchComposition(groups: IndividualTransaction[][]): IndividualTransaction[] {
    const optimizedBatch: IndividualTransaction[] = [];
    let remainingCapacity = this.batchConfig.maxBatchSize;

    for (const group of groups) {
      if (remainingCapacity <= 0) break;
      
      const groupSize = Math.min(group.length, remainingCapacity);
      optimizedBatch.push(...group.slice(0, groupSize));
      remainingCapacity -= groupSize;
    }

    return optimizedBatch;
  }

  private calculateBatchFee(transactions: IndividualTransaction[]): number {
    // Batch fees are typically lower than individual transaction fees
    const baseFee = 0.00001; // Base fee in lumens
    const individualFees = transactions.reduce((sum, tx) => sum + tx.fee, 0);
    
    // Apply batch discount
    const batchSize = transactions.length;
    const discountRate = this.calculateBatchDiscount(batchSize);
    
    return Math.max(baseFee, individualFees * (1 - discountRate));
  }

  private calculateBatchDiscount(batchSize: number): number {
    // Larger batches get better discounts
    if (batchSize >= 100) return 0.5; // 50% discount
    if (batchSize >= 50) return 0.3;  // 30% discount
    if (batchSize >= 20) return 0.2;  // 20% discount
    if (batchSize >= 10) return 0.1;  // 10% discount
    return 0; // No discount for small batches
  }

  private calculateSavings(individualFees: number, batchFee: number): BatchSavings {
    const savings = individualFees - batchFee;
    const savingsPercentage = individualFees > 0 ? (savings / individualFees) * 100 : 0;

    return {
      totalIndividualFees: individualFees,
      batchFee,
      savings,
      savingsPercentage,
    };
  }

  private async submitBatchToBlockchain(batch: BatchTransaction): Promise<BatchTransaction> {
    // Mock blockchain submission
    // In a real implementation, this would interact with the Stellar network
    
    const processingTime = Math.random() * 3000 + 1000; // 1-4 seconds
    
    await new Promise(resolve => setTimeout(resolve, processingTime));
    
    const success = Math.random() > 0.05; // 95% success rate
    
    return {
      ...batch,
      status: success ? 'completed' : 'failed',
    };
  }

  private updateMetrics(batch: BatchTransaction): void {
    this.batchMetrics.totalBatches++;
    this.batchMetrics.totalTransactions += batch.transactions.length;
    this.batchMetrics.averageBatchSize = 
      this.batchMetrics.totalTransactions / this.batchMetrics.totalBatches;
    
    if (batch.savings) {
      this.batchMetrics.totalSavings += batch.savings.savings;
      this.batchMetrics.averageSavingsPercentage = 
        (this.batchMetrics.totalSavings / this.batchMetrics.totalBatches) / 100;
    }
  }

  private validateBatchConfig(config: BatchingConfig): void {
    if (config.maxBatchSize <= 0) {
      throw new Error('maxBatchSize must be greater than 0');
    }
    
    if (config.minBatchSize <= 0) {
      throw new Error('minBatchSize must be greater than 0');
    }
    
    if (config.minBatchSize > config.maxBatchSize) {
      throw new Error('minBatchSize cannot be greater than maxBatchSize');
    }
    
    if (config.batchTimeout <= 0) {
      throw new Error('batchTimeout must be greater than 0');
    }
  }

  private startBatchProcessor(): void {
    // Process batches at regular intervals
    setInterval(async () => {
      if (this.pendingTransactions.length >= this.batchConfig.minBatchSize) {
        await this.processBatch();
      }
    }, this.batchConfig.batchTimeout);
  }

  private generateTransactionId(): string {
    return `tx_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  private generateBatchId(): string {
    return `batch_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  async getPendingTransactions(): Promise<IndividualTransaction[]> {
    return [...this.pendingTransactions];
  }

  async getBatchHistory(limit: number = 50): Promise<BatchTransaction[]> {
    // In a real implementation, query from database
    return [];
  }

  async cancelBatch(batchId: string): Promise<boolean> {
    // In a real implementation, cancel the batch if not yet processed
    return false;
  }

  async retryFailedBatch(batchId: string): Promise<BatchTransaction | null> {
    // In a real implementation, retry failed transactions
    return null;
  }
}
