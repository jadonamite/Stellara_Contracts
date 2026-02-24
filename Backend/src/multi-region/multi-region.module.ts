import { Module } from '@nestjs/common';
import { MultiRegionController } from './multi-region.controller';
import { MultiRegionService } from './multi-region.service';
import { FailoverService } from './failover.service';
import { DisasterRecoveryService } from './disaster-recovery.service';
import { DataSyncService } from './data-sync.service';

@Module({
  controllers: [MultiRegionController],
  providers: [
    MultiRegionService,
    FailoverService,
    DisasterRecoveryService,
    DataSyncService,
  ],
  exports: [
    MultiRegionService,
    FailoverService,
    DisasterRecoveryService,
    DataSyncService,
  ],
})
export class MultiRegionModule {}
