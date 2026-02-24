import {
  WebSocketGateway,
  WebSocketServer,
  OnGatewayConnection,
  OnGatewayDisconnect,
  SubscribeMessage,
} from '@nestjs/websockets';
import { Server, Socket } from 'socket.io';
import { PresenceService } from './presence.service';
import { EventStorageService } from '../stellar-monitor/services/event-storage.service';
import { EventType } from '../stellar-monitor/types/stellar.types';

@WebSocketGateway({
  cors: { origin: '*' },
})
export class WebsocketGateway
  implements OnGatewayConnection, OnGatewayDisconnect
{
  @WebSocketServer()
  server: Server;

  private eventSubscriptions: Map<string, { eventTypes?: EventType[]; contractIds?: string[]; accounts?: string[] }> = new Map();

  constructor(
    private readonly presenceService: PresenceService,
    private readonly eventStorageService: EventStorageService,
  ) {}

  async handleConnection(client: Socket) {
    const userId = client.handshake.auth.userId;
    await this.presenceService.userConnected(userId, client.id);
  }

  async handleDisconnect(client: Socket) {
    const userId = client.handshake.auth.userId;
    await this.presenceService.userDisconnected(userId);
  }

  @SubscribeMessage('join-room')
  async joinRoom(client: Socket, roomId: string) {
    const userId = client.handshake.auth.userId;

    await this.presenceService.joinRoom(userId, roomId);
    client.join(roomId);

    this.server.to(roomId).emit('presence:update', {
      roomId,
      users: await this.presenceService.getRoomUsers(roomId),
    });
  }

  @SubscribeMessage('message')
  async handleMessage(
    client: Socket,
    payload: { roomId: string; message: string },
  ) {
    this.server.to(payload.roomId).emit('message', payload);
  }

  @SubscribeMessage('subscribe-events')
  async subscribeToEvents(
    client: Socket,
    filters: { eventTypes?: EventType[]; contractIds?: string[]; accounts?: string[] },
  ) {
    this.eventSubscriptions.set(client.id, filters);
    client.join('events');
    
    client.emit('subscription-confirmed', { 
      message: 'Subscribed to events',
      filters 
    });
  }

  @SubscribeMessage('unsubscribe-events')
  async unsubscribeFromEvents(client: Socket) {
    this.eventSubscriptions.delete(client.id);
    client.leave('events');
    
    client.emit('subscription-removed', { 
      message: 'Unsubscribed from events' 
    });
  }

  // Method to broadcast events to subscribed clients
  async broadcastEvent(event: any) {
    const subscribedClients = await this.server.in('events').fetchSockets();
    
    for (const client of subscribedClients) {
      const subscription = this.eventSubscriptions.get(client.id);
      
      if (!subscription || this.isEventRelevantForSubscription(event, subscription)) {
        client.emit('stellar-event', event);
      }
    }
  }

  private isEventRelevantForSubscription(
    event: any, 
    subscription: { eventTypes?: EventType[]; contractIds?: string[]; accounts?: string[] }
  ): boolean {
    // Check event type filter
    if (subscription.eventTypes && subscription.eventTypes.length > 0) {
      if (!subscription.eventTypes.includes(event.eventType)) {
        return false;
      }
    }

    // Check contract ID filter
    if (subscription.contractIds && subscription.contractIds.length > 0) {
      const eventContractId = event.payload.contractId || event.sourceAccount;
      if (!subscription.contractIds.includes(eventContractId)) {
        return false;
      }
    }

    // Check account filter
    if (subscription.accounts && subscription.accounts.length > 0) {
      if (!subscription.accounts.includes(event.sourceAccount)) {
        return false;
      }
    }

    return true;
  }
}
