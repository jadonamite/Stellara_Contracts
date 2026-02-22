import { Module } from '@nestjs/common';
import { CqrsModule } from '@nestjs/cqrs';
import { TypeOrmModule } from '@nestjs/typeorm';
import { StakingService } from './staking.service';
import { User } from '../users/entities/user.entity';
import { Transaction } from '../transactions/entities/transaction.entity';

@Module({
  imports: [CqrsModule, TypeOrmModule.forFeature([User, Transaction])],
  providers: [StakingService],
  exports: [StakingService],
})
export class StakingModule {}a
