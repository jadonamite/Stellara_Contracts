import { Module } from '@nestjs/common';
import { StructuredLogger } from './structured-logger.service';

@Module({
  providers: [
    {
      provide: 'LoggerService',
      useClass: StructuredLogger,
    },
  ],
  exports: ['LoggerService'],
})
export class LoggingModule {}
