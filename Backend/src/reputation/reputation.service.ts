import { Injectable, NotFoundException } from '@nestjs/common';
import { InjectRepository } from '@nestjs/typeorm';
import { User } from 'src/auth/entities/user.entity';
import { ReputationLog } from './entities/reputation-log.entity';
import { Repository } from 'typeorm';
import {
  levelFromXp,
  rankFromReputation,
  REPUTATION_POINTS,
  ReputationAction,
  XP_POINTS,
} from './types/reputation.types';

@Injectable()
export class ReputationService {
  constructor(
    @InjectRepository(User) private userRepo: Repository<User>,
    @InjectRepository(ReputationLog) private logRepo: Repository<ReputationLog>,
  ) {}

  async applyAction(userId: number, action: ReputationAction) {
    const user = await this.userRepo.findOneBy({ id: userId });
    if (!user) throw new NotFoundException('User not found');

    const repChange = REPUTATION_POINTS[action] ?? 0;
    const xpChange = XP_POINTS[action] ?? 0;

    user.reputation += repChange;
    user.totalXp += xpChange;

    // Update level
    const newLevel = levelFromXp(user.totalXp);
    user.level = newLevel;

    // Update rank
    user.rank = rankFromReputation(user.reputation);

    await this.userRepo.save(user);

    await this.logRepo.save({
      user,
      action,
      reputationChange: repChange,
      xpChange,
    });

    return user;
  }
}
