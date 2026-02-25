import { Module, Logger } from '@nestjs/common';
import { TypeOrmModule } from '@nestjs/typeorm';
import { ConfigModule, ConfigService } from '@nestjs/config';
import { APP_GUARD } from '@nestjs/core';

import { AppController } from './app.controller';
import { AppService } from './app.service';

// logging and error handling
// import { LoggingModule } from './logging/logging.module';         // TODO: module not yet created
// import { StructuredLogger } from './logging/structured-logger.service'; // TODO: module not yet created

import { RedisModule } from './redis/redis.module';
import { VoiceModule } from './voice/voice.module';
import { StellarMonitorModule } from './stellar-monitor/stellar-monitor.module';
import { WorkflowModule } from './workflow/workflow.module';
import { QueueModule } from './queue/queue.module';
import { AuthModule } from './auth/auth.module';
import { MarketDataModule } from './market-data/market-data.module';
// import { AnalyticsModule } from './analytics/analytics.module';   // TODO: module not yet created
import { PersonalizationModule } from './personalization/personalization.module';

import { RolesGuard } from './guards/roles.guard';

import { Workflow } from './workflow/entities/workflow.entity';
import { WorkflowStep } from './workflow/entities/workflow-step.entity';
import { User } from './auth/entities/user.entity';
import { WalletBinding } from './auth/entities/wallet-binding.entity';
import { LoginNonce } from './auth/entities/login-nonce.entity';
import { RefreshToken } from './auth/entities/refresh-token.entity';
import { ApiToken } from './auth/entities/api-token.entity';
import { AuditModule } from './audit/audit.module';
import { AuditLog } from './audit/audit.entity';
// import { GdprModule } from './gdpr/gdpr.module';                  // TODO: module not yet created
// import { Consent } from './gdpr/entities/consent.entity';         // TODO: module not yet created
import { VoiceJob } from './voice/entities/voice-job.entity';
import { ThrottleModule } from './throttle/throttle.module';
// import { TenantModule } from './tenancy/tenant.module';                       // TODO: module not yet created
// import { Tenant } from './tenancy/entities/tenant.entity';                    // TODO: module not yet created
// import { TenantConfig } from './tenancy/entities/tenant-config.entity';       // TODO: module not yet created
// import { TenantUsage } from './tenancy/entities/tenant-usage.entity';         // TODO: module not yet created
// import { TenantInvitation } from './tenancy/entities/tenant-invitation.entity'; // TODO: module not yet created
import { StellarEvent } from './stellar-monitor/entities/stellar-event.entity';
import { WebhookConsumer } from './stellar-monitor/entities/webhook-consumer.entity';
// import { BlockchainModule } from './blockchain/blockchain.module'; // TODO: module not yet created
import { WebsocketModule } from './websocket/websocket.module';
// import { AnalyticsMetric } from './analytics/entities/analytics-metric.entity'; // TODO: module not yet created
// import { AnalyticsAlert } from './analytics/entities/analytics-alert.entity';   // TODO: module not yet created
import { UserEvent } from './personalization/entities/user-event.entity';
import { PersonalizationRule } from './personalization/entities/personalization-rule.entity';
import { Experiment } from './personalization/entities/experiment.entity';
import { ExperimentAssignment } from './personalization/entities/experiment-assignment.entity';


@Module({
  imports: [
    // LoggingModule, // TODO: module not yet created

    ConfigModule.forRoot({
      isGlobal: true,
    }),

    TypeOrmModule.forRootAsync({
      imports: [ConfigModule],
      inject: [ConfigService],
      useFactory: (configService: ConfigService) => {
        const dbType = configService.get('DB_TYPE') || 'sqlite';

        const baseConfig: any = {
          type: dbType,
          synchronize: configService.get('NODE_ENV') === 'development',
          logging: configService.get('NODE_ENV') === 'development',
          entities: [
            Workflow,
            WorkflowStep,
            User,
            WalletBinding,
            LoginNonce,
            RefreshToken,
            ApiToken,
            AuditLog,
            // Consent,          // TODO: module not yet created
            VoiceJob,
            // Tenant,           // TODO: module not yet created
            // TenantConfig,     // TODO: module not yet created
            // TenantUsage,      // TODO: module not yet created
            // TenantInvitation, // TODO: module not yet created
            // AnalyticsMetric,  // TODO: module not yet created
            // AnalyticsAlert,   // TODO: module not yet created
            UserEvent,
            PersonalizationRule,
            Experiment,
            ExperimentAssignment,
            StellarEvent,
            WebhookConsumer,
          ],
        };

        if (dbType === 'sqlite') {
          baseConfig.database = configService.get('DB_DATABASE') || './stellar-events.db';
        } else {
          baseConfig.host = configService.get('DB_HOST') || 'localhost';
          baseConfig.port = configService.get('DB_PORT') || 5432;
          baseConfig.username = configService.get('DB_USERNAME') || 'postgres';
          baseConfig.password = configService.get('DB_PASSWORD') || 'password';
          baseConfig.database = configService.get('DB_DATABASE') || 'stellara_workflows';
        }

        return baseConfig;
      },
    }),

    RedisModule,
    AuthModule,
    VoiceModule,
    StellarMonitorModule,
    WorkflowModule,
    QueueModule,
    MarketDataModule,
    AuditModule,
    // GdprModule,       // TODO: module not yet created
    ThrottleModule,
    // TenantModule,     // TODO: module not yet created
    // AnalyticsModule,  // TODO: module not yet created
    PersonalizationModule,
    // BlockchainModule, // TODO: module not yet created
    WebsocketModule,
  ],

  controllers: [AppController],

  providers: [
    AppService,
    {
      provide: APP_GUARD,
      useClass: RolesGuard,
    },
    // { provide: Logger, useClass: StructuredLogger }, // TODO: module not yet created
  ],
})
export class AppModule {}