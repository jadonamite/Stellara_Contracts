import { Injectable, UnauthorizedException } from '@nestjs/common';
import { JwtService as NestJwtService } from '@nestjs/jwt';
import { ConfigService } from '@nestjs/config';
import { InjectRepository } from '@nestjs/typeorm';
import { Repository, LessThan } from 'typeorm';
import { RefreshToken } from '../entities/refresh-token.entity';
import { User } from '../entities/user.entity';
import { randomUUID } from 'crypto';
import { AuditService } from '../../audit/audit.service';
import { Cron, CronExpression } from '@nestjs/schedule';
import { AuditEvent } from '../../audit/audit.event';

export interface JwtPayload {
  sub: string;
  walletId?: string;
  iat?: number;
  exp?: number;
}

@Injectable()
export class JwtAuthService {
  constructor(
    private readonly jwtService: NestJwtService,
    private readonly configService: ConfigService,
    @InjectRepository(RefreshToken)
    private readonly refreshTokenRepository: Repository<RefreshToken>,
    @InjectRepository(User)
    private readonly userRepository: Repository<User>,
    private readonly auditService: AuditService,
  ) {}

  async generateAccessToken(
    userId: string,
    walletId?: string,
  ): Promise<string> {
    const payload: JwtPayload = {
      sub: userId,
      walletId,
    };

    const expiresIn = this.configService.get('JWT_ACCESS_EXPIRATION', '15m') as `${number}${'s' | 'm' | 'h' | 'd'}`;

    return this.jwtService.sign(payload, { expiresIn });
  }

  async generateRefreshToken(
    userId: string,
  ): Promise<{ token: string; id: string; expiresAt: Date }> {
    const token = randomUUID();
    const expirationDays = parseInt(
      this.configService.get<string>('JWT_REFRESH_EXPIRATION_DAYS', '7'),
      10,
    );
    const expiresAt = new Date();
    expiresAt.setDate(expiresAt.getDate() + expirationDays);

    const refreshToken = this.refreshTokenRepository.create({
      token,
      userId,
      expiresAt,
      revoked: false,
    });

    const saved = await this.refreshTokenRepository.save(refreshToken);

    await this.auditService.logAction(
      'REFRESH_TOKEN_CREATED',
      userId,
      saved.id,
      { expiresAt: saved.expiresAt },
    );

    return {
      token: saved.token,
      id: saved.id,
      expiresAt: saved.expiresAt,
    };
  }

  async validateAccessToken(token: string): Promise<JwtPayload> {
    try {
      const payload = this.jwtService.verify(token);
      return payload as JwtPayload;
    } catch {
      throw new UnauthorizedException('Invalid or expired access token');
    }
  }

  async refreshAccessToken(
    refreshToken: string,
  ): Promise<{ accessToken: string; newRefreshToken: string }> {
    const tokenRecord = await this.refreshTokenRepository.findOne({
      where: { token: refreshToken },
      relations: ['user'],
    });

    if (!tokenRecord) {
      throw new UnauthorizedException('Invalid refresh token');
    }

    if (tokenRecord.revoked) {
      // Potential token theft — revoke all tokens for this user immediately
      await this.revokeAllUserRefreshTokens(tokenRecord.userId);
      await this.auditService.logAction(
        'REFRESH_TOKEN_REUSE_DETECTED',
        tokenRecord.userId,
        tokenRecord.id,
        { revokedAt: new Date() },
      );
      throw new UnauthorizedException(
        'Refresh token has been revoked. All sessions invalidated for security.',
      );
    }

    if (new Date() > tokenRecord.expiresAt) {
      throw new UnauthorizedException('Refresh token expired');
    }

    if (!tokenRecord.user.isActive) {
      throw new UnauthorizedException('User account is inactive');
    }

    // Rotate: revoke old token before issuing new one
    await this.revokeRefreshToken(tokenRecord.id);

    // Issue new token pair
    const accessToken = await this.generateAccessToken(tokenRecord.userId);
    const newRefreshTokenData = await this.generateRefreshToken(
      tokenRecord.userId,
    );

    await this.auditService.logAction(
      'ACCESS_TOKEN_REFRESHED',
      tokenRecord.userId,
      tokenRecord.id,
    );

    return {
      accessToken,
      newRefreshToken: newRefreshTokenData.token,
    };
  }

  async revokeRefreshToken(tokenId: string): Promise<void> {
    await this.refreshTokenRepository.update(
      { id: tokenId },
      {
        revoked: true,
        revokedAt: new Date(),
      },
    );

    await this.auditService.logAction(
      'REFRESH_TOKEN_REVOKED',
      tokenId,
      tokenId,
    );
  }

  async revokeAllUserRefreshTokens(userId: string): Promise<void> {
    await this.refreshTokenRepository.update(
      { userId, revoked: false },
      {
        revoked: true,
        revokedAt: new Date(),
      },
    );

    await this.auditService.logAction(
      'ALL_REFRESH_TOKENS_REVOKED',
      userId,
      userId,
      { reason: 'logout_or_security_event' },
    );
  }

  async getUserFromToken(token: string): Promise<User> {
    const payload = await this.validateAccessToken(token);
    const user = await this.userRepository.findOne({
      where: { id: payload.sub },
    });

    if (!user) {
      throw new UnauthorizedException('User not found');
    }

    if (!user.isActive) {
      throw new UnauthorizedException('User account is inactive');
    }

    return user;
  }

  /**
   * Runs every hour to purge refresh tokens that are expired or revoked
   * older than 30 days. Keeps the table lean in production.
   */
  @Cron(CronExpression.EVERY_HOUR)
  async cleanupExpiredRefreshTokens(): Promise<void> {
    const now = new Date();
    const thirtyDaysAgo = new Date();
    thirtyDaysAgo.setDate(thirtyDaysAgo.getDate() - 30);

    // Delete tokens that are expired
    await this.refreshTokenRepository.delete({
      expiresAt: LessThan(now),
    });

    // Delete tokens that were revoked more than 30 days ago
    await this.refreshTokenRepository.delete({
      revoked: true,
      revokedAt: LessThan(thirtyDaysAgo),
    });
  }
}