import { Module } from '@nestjs/common';
import { WebsocketGateway } from './websocket.gateway';
import { PresenceService } from './presence.service';
import { StellarMonitorModule } from '../stellar-monitor/stellar-monitor.module';

@Module({
  imports: [StellarMonitorModule],
  providers: [WebsocketGateway, PresenceService],
  exports: [WebsocketGateway],
})
export class WebsocketModule {}
