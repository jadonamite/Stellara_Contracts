import { Module } from '@nestjs/common';
import { TypeOrmModule } from '@nestjs/typeorm';
import { AnalyticsMetric } from './entities/analytics-metric.entity';
import { AnalyticsAlert } from './entities/analytics-alert.entity';

@Module({
  imports: [TypeOrmModule.forFeature([AnalyticsMetric, AnalyticsAlert])],
  providers: [],
  exports: [TypeOrmModule],
})
export class AnalyticsModule {}
