import { Module } from '@nestjs/common';
import { VoiceGateway } from './voice.gateway';
import { VoiceSessionService } from './services/voice-session.service';
import { ConversationStateMachineService } from './services/conversation-state-machine.service';
import { StreamingResponseService } from './services/streaming-response.service';
import { SessionCleanupService } from './services/session-cleanup.service';
import { LlmService } from './services/llm.service';
import { QuotaService } from './services/quota.service';
import { LlmCacheService } from './services/llm-cache.service';
import { RedisModule } from '../redis/redis.module';

@Module({
  imports: [RedisModule],
  providers: [
    VoiceGateway,
    VoiceSessionService,
    ConversationStateMachineService,
    StreamingResponseService,
    SessionCleanupService,
    LlmService,
    QuotaService,
    LlmCacheService,
  ],
  exports: [
    VoiceSessionService,
    ConversationStateMachineService,
    StreamingResponseService,
    LlmService,
    QuotaService,
    LlmCacheService,
  ],
})
export class VoiceModule {}
