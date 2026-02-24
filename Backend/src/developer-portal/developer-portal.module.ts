import { Module } from '@nestjs/common';
import { DeveloperPortalController } from './developer-portal.controller';
import { DeveloperPortalService } from './developer-portal.service';
import { ApiDocumentationService } from './api-documentation.service';
import { SdkGenerationService } from './sdk-generation.service';
import { OnboardingService } from './onboarding.service';
import { AnalyticsService } from './analytics.service';

@Module({
  controllers: [DeveloperPortalController],
  providers: [
    DeveloperPortalService,
    ApiDocumentationService,
    SdkGenerationService,
    OnboardingService,
    AnalyticsService,
  ],
  exports: [
    DeveloperPortalService,
    ApiDocumentationService,
    SdkGenerationService,
    OnboardingService,
    AnalyticsService,
  ],
})
export class DeveloperPortalModule {}
