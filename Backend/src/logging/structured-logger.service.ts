import { Injectable, LoggerService } from '@nestjs/common';
import { Logger } from '@nestjs/common';

@Injectable()
export class StructuredLogger implements LoggerService {
  private readonly logger = new Logger('StructuredLogger');

  log(message: any, context?: string) {
    const logEntry = {
      timestamp: new Date().toISOString(),
      level: 'log',
      message,
      context: context || 'App',
      ...this.getExtraContext(),
    };
    this.logger.log(JSON.stringify(logEntry));
  }

  error(message: any, trace?: string, context?: string) {
    const logEntry = {
      timestamp: new Date().toISOString(),
      level: 'error',
      message,
      trace,
      context: context || 'App',
      ...this.getExtraContext(),
    };
    this.logger.error(JSON.stringify(logEntry));
  }

  warn(message: any, context?: string) {
    const logEntry = {
      timestamp: new Date().toISOString(),
      level: 'warn',
      message,
      context: context || 'App',
      ...this.getExtraContext(),
    };
    this.logger.warn(JSON.stringify(logEntry));
  }

  debug(message: any, context?: string) {
    const logEntry = {
      timestamp: new Date().toISOString(),
      level: 'debug',
      message,
      context: context || 'App',
      ...this.getExtraContext(),
    };
    this.logger.debug(JSON.stringify(logEntry));
  }

  verbose(message: any, context?: string) {
    const logEntry = {
      timestamp: new Date().toISOString(),
      level: 'verbose',
      message,
      context: context || 'App',
      ...this.getExtraContext(),
    };
    this.logger.verbose(JSON.stringify(logEntry));
  }

  private getExtraContext(): Record<string, any> {
    // Add any extra context you want here
    return {};
  }
}
