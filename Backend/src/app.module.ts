import { Module } from '@nestjs/common';
import { TypeOrmModule } from '@nestjs/typeorm';
import { ConfigModule, ConfigService } from '@nestjs/config';
import { AppController } from './app.controller';
import { AppService } from './app.service';
import { RedisModule } from './redis/redis.module';
import { VoiceModule } from './voice/voice.module';
import { DatabaseModule } from './database/database.module';
import { StellarMonitorModule } from './stellar-monitor/stellar-monitor.module';

@Module({
  imports: [
    DatabaseModule,
    RedisModule,
    VoiceModule,
    StellarMonitorModule,
import { WorkflowModule } from './workflow/workflow.module';
import { Workflow } from './workflow/entities/workflow.entity';
import { WorkflowStep } from './workflow/entities/workflow-step.entity';

@Module({
  imports: [
    ConfigModule.forRoot({
      isGlobal: true,
    }),
    TypeOrmModule.forRootAsync({
      imports: [ConfigModule],
      useFactory: (configService: ConfigService) => ({
        type: 'postgres',
        host: configService.get('DB_HOST') || 'localhost',
        port: configService.get('DB_PORT') || 5432,
        username: configService.get('DB_USERNAME') || 'postgres',
        password: configService.get('DB_PASSWORD') || 'password',
        database: configService.get('DB_DATABASE') || 'stellara_workflows',
        entities: [Workflow, WorkflowStep],
        synchronize: configService.get('NODE_ENV') === 'development',
        logging: configService.get('NODE_ENV') === 'development',
      }),
      inject: [ConfigService],
    }),
    RedisModule,
    VoiceModule,
    WorkflowModule,
  ],
  controllers: [AppController],
  providers: [AppService],
})
export class AppModule {}
