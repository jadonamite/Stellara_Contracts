import { Module } from '@nestjs/common';
import { TransactionOptimizationController } from './transaction-optimization.controller';
import { TransactionOptimizationService } from './transaction-optimization.service';
import { BatchingService } from './batching.service';
import { FeeOptimizationService } from './fee-optimization.service';
import { PrioritizationService } from './prioritization.service';

@Module({
  controllers: [TransactionOptimizationController],
  providers: [
    TransactionOptimizationService,
    BatchingService,
    FeeOptimizationService,
    PrioritizationService,
  ],
  exports: [
    TransactionOptimizationService,
    BatchingService,
    FeeOptimizationService,
    PrioritizationService,
  ],
})
export class TransactionOptimizationModule {}
