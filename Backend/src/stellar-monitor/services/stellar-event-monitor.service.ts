import {
  Injectable,
  Logger,
  OnModuleInit,
  OnModuleDestroy,
} from '@nestjs/common';
import { Horizon, SorobanRpc } from '@stellar/stellar-sdk';
import { v4 as uuidv4 } from 'uuid';
import { EventStorageService } from './event-storage.service';
import { WebhookDeliveryService } from './webhook-delivery.service';
import { StellarEvent } from '../entities/stellar-event.entity';
import { EventType } from '../types/stellar.types';

interface HorizonPaymentOperation {
  id: string;
  paging_token: string;
  source_account: string;
  type_i: number;
  type: string;
  created_at: string;
  transaction_hash: string;
  asset_type: string;
  asset_code?: string;
  asset_issuer?: string;
  from: string;
  to: string;
  amount: string;
  transaction_attr: {
    ledger: number;
    memo?: string;
    memo_type?: string;
  };
}

interface HorizonManageOfferOperation {
  id: string;
  paging_token: string;
  source_account: string;
  type_i: number;
  type: string;
  created_at: string;
  transaction_hash: string;
  offer_id?: string;
  amount: string;
  price: string;
  selling_asset_type: string;
  selling_asset_code?: string;
  selling_asset_issuer?: string;
  buying_asset_type: string;
  buying_asset_code?: string;
  buying_asset_issuer?: string;
  transaction_attr: {
    ledger: number;
  };
}

@Injectable()
export class StellarEventMonitorService
  implements OnModuleInit, OnModuleDestroy
{
  private readonly logger = new Logger(StellarEventMonitorService.name);
  private horizonServer: Horizon.Server;
  private sorobanServer: SorobanRpc.Server;
  private paymentStream: (() => void) | null = null;
  private offerStream: (() => void) | null = null;
  private contractEventStream: (() => void) | null = null;
  private isMonitoring = false;
  private lastLedgerSequence = 0;
  private lastEventId = '';

  constructor(
    private readonly eventStorageService: EventStorageService,
    private readonly webhookDeliveryService: WebhookDeliveryService,
  ) {
    const horizonUrl =
      process.env.HORIZON_URL || 'https://horizon-testnet.stellar.org';
    const sorobanUrl =
      process.env.SOROBAN_RPC_URL || 'https://soroban-testnet.stellar.org';
    
    this.horizonServer = new Horizon.Server(horizonUrl);
    this.sorobanServer = new SorobanRpc.Server(sorobanUrl);
    this.logger.log(`Initialized Horizon server at ${horizonUrl}`);
    this.logger.log(`Initialized Soroban RPC server at ${sorobanUrl}`);
  }

  async onModuleInit() {
    if (process.env.STELLAR_MONITOR_ENABLED !== 'false') {
      await this.startMonitoring();
    }
  }

  async onModuleDestroy() {
    await this.stopMonitoring();
  }

  async startMonitoring(): Promise<void> {
    if (this.isMonitoring) {
      this.logger.warn('Monitoring is already running');
      return;
    }

    try {
      this.isMonitoring = true;

      // Get current ledger to start from
      const ledger = await this.horizonServer
        .ledgers()
        .order('desc')
        .limit(1)
        .call();
      this.lastLedgerSequence = ledger.records[0].sequence;

      // Initialize contract event cursor
      this.lastEventId = '';

      this.logger.log(
        `Starting monitoring from ledger ${this.lastLedgerSequence}`,
      );

      // Start streaming payments
      this.startPaymentStream();

      // Start streaming offers
      this.startOfferStream();

      // Start streaming contract events
      this.startContractEventStream();

      this.logger.log('Stellar event monitoring started successfully');
    } catch (error) {
      this.logger.error(
        `Failed to start monitoring: ${error.message}`,
        error.stack,
      );
      this.isMonitoring = false;
      throw error;
    }
  }

  async stopMonitoring(): Promise<void> {
    if (!this.isMonitoring) {
      return;
    }

    this.logger.log('Stopping Stellar event monitoring...');

    try {
      if (this.paymentStream) {
        this.paymentStream();
        this.paymentStream = null;
      }

      if (this.offerStream) {
        this.offerStream();
        this.offerStream = null;
      }

      if (this.contractEventStream) {
        this.contractEventStream();
        this.contractEventStream = null;
      }

      this.isMonitoring = false;
      this.logger.log('Stellar event monitoring stopped');
    } catch (error) {
      this.logger.error(
        `Error stopping monitoring: ${error.message}`,
        error.stack,
      );
    }
  }

  private startPaymentStream(): void {
    this.paymentStream = this.horizonServer
      .payments()
      .cursor('now')
      .stream({
        onmessage: (payment: any) => {
          this.handlePaymentEvent(payment).catch((error) => {
            this.logger.error(
              `Error handling payment event: ${error.message}`,
              error.stack,
            );
          });
        },
        onerror: (event: MessageEvent) => {
          const error = event as unknown as Error;
          this.logger.error(
            `Payment stream error: ${error.message}`,
            error.stack,
          );
          // Attempt to restart the stream
          setTimeout(() => {
            if (this.isMonitoring) {
              this.logger.log('Attempting to restart payment stream...');
              this.startPaymentStream();
            }
          }, 5000);
        },
      });
  }

  private startOfferStream(): void {
    this.offerStream = this.horizonServer
      .operations()
      .cursor('now')
      .stream({
        onmessage: (offer: any) => {
          // Filter for manage offer operations
          if (
            offer.type === 'manage_sell_offer' ||
            offer.type === 'manage_buy_offer'
          ) {
            this.handleOfferEvent(offer).catch((error) => {
              this.logger.error(
                `Error handling offer event: ${error.message}`,
                error.stack,
              );
            });
          }
        },
        onerror: (event: MessageEvent) => {
          const error = event as unknown as Error;
          this.logger.error(
            `Offer stream error: ${error.message}`,
            error.stack,
          );
          // Attempt to restart the stream
          setTimeout(() => {
            if (this.isMonitoring) {
              this.logger.log('Attempting to restart offer stream...');
              this.startOfferStream();
            }
          }, 5000);
        },
      });
  }

  private startContractEventStream(): void {
    this.contractEventStream = this.sorobanServer
      .getEvents({
        startLedger: this.lastLedgerSequence,
        cursor: this.lastEventId || undefined,
        limit: 100,
      })
      .then((response) => {
        if (response.events && response.events.length > 0) {
          for (const event of response.events) {
            this.handleContractEvent(event).catch((error) => {
              this.logger.error(
                `Error handling contract event: ${error.message}`,
                error.stack,
              );
            });
          }
          
          // Update cursor
          const lastEvent = response.events[response.events.length - 1];
          this.lastEventId = lastEvent.id;
          this.lastLedgerSequence = lastEvent.ledger;
        }
      })
      .catch((error) => {
        this.logger.error(
          `Contract event stream error: ${error.message}`,
          error.stack,
        );
        // Attempt to restart the stream
        setTimeout(() => {
          if (this.isMonitoring) {
            this.logger.log('Attempting to restart contract event stream...');
            this.startContractEventStream();
          }
        }, 5000);
      });
  }

  private async handlePaymentEvent(payment: any): Promise<void> {
    try {
      const eventData = {
        id: uuidv4(),
        eventType: EventType.PAYMENT,
        ledgerSequence: payment.transaction_attr.ledger,
        timestamp: new Date(payment.created_at).toISOString(),
        transactionHash: payment.transaction_hash,
        sourceAccount: payment.from,
        payload: {
          amount: payment.amount,
          assetType: payment.asset_type,
          assetCode: payment.asset_code,
          assetIssuer: payment.asset_issuer,
          from: payment.from,
          to: payment.to,
          memo: payment.transaction_attr.memo,
          memoType: payment.transaction_attr.memo_type,
        },
      };

      const savedEvent = await this.eventStorageService.saveEvent(eventData);
      await this.webhookDeliveryService.queueEventForDelivery(savedEvent);

      this.logger.debug(
        `Processed payment event ${savedEvent.id} from ${payment.from} to ${payment.to}`,
      );
    } catch (error) {
      this.logger.error(
        `Failed to process payment event: ${error.message}`,
        error.stack,
      );
    }
  }

  private async handleOfferEvent(offer: any): Promise<void> {
    try {
      const eventData = {
        id: uuidv4(),
        eventType: EventType.OFFER,
        ledgerSequence: offer.transaction_attr.ledger,
        timestamp: new Date(offer.created_at).toISOString(),
        transactionHash: offer.transaction_hash,
        sourceAccount: offer.source_account,
        payload: {
          offerId: offer.offer_id?.toString(),
          seller: offer.source_account,
          sellingAssetType: offer.selling_asset_type,
          sellingAssetCode: offer.selling_asset_code,
          sellingAssetIssuer: offer.selling_asset_issuer,
          buyingAssetType: offer.buying_asset_type,
          buyingAssetCode: offer.buying_asset_code,
          buyingAssetIssuer: offer.buying_asset_issuer,
          amount: offer.amount,
          price: offer.price,
          type: this.determineOfferType(offer),
        },
      };

      const savedEvent = await this.eventStorageService.saveEvent(eventData);
      await this.webhookDeliveryService.queueEventForDelivery(savedEvent);

      this.logger.debug(
        `Processed offer event ${savedEvent.id} from ${offer.source_account}`,
      );
    } catch (error) {
      this.logger.error(
        `Failed to process offer event: ${error.message}`,
        error.stack,
      );
    }
  }

  private async handleContractEvent(event: SorobanRpc.Api.EventResponse): Promise<void> {
    try {
      // Decode the event data
      const topics = event.topic.map(t => SorobanRpc.xdr.ScVal.fromXDR(t, 'base64'));
      const value = SorobanRpc.xdr.ScVal.fromXDR(event.value, 'base64');
      
      // Extract topic string
      const topicString = this.decodeScValToString(topics[0]);
      
      // Determine event type based on topic
      const eventType = this.mapTopicToEventType(topicString);
      
      if (!eventType) {
        this.logger.debug(`Unknown contract event topic: ${topicString}`);
        return;
      }

      const eventData = {
        id: uuidv4(),
        eventType,
        ledgerSequence: event.ledger,
        timestamp: event.ledgerClosedAt,
        transactionHash: event.txHash,
        sourceAccount: event.contractId, // Contract ID as source for contract events
        payload: {
          contractId: event.contractId,
          topics: topics.map(t => this.decodeScValToString(t)),
          data: this.decodeEventValue(value),
          topic: topicString,
          eventIndex: event.eventIndex,
        },
      };

      const savedEvent = await this.eventStorageService.saveEvent(eventData);
      await this.webhookDeliveryService.queueEventForDelivery(savedEvent);

      this.logger.debug(
        `Processed contract event ${savedEvent.id} of type ${eventType} from contract ${event.contractId}`,
      );
    } catch (error) {
      this.logger.error(
        `Failed to process contract event: ${error.message}`,
        error.stack,
      );
    }
  }

  private decodeScValToString(scVal: SorobanRpc.xdr.ScVal): string {
    const scValType = scVal.switch().name;
    if (scValType === 'scvSymbol') {
      return scVal.sym().toString();
    }
    return scVal.toXDR('base64'); // Fallback to base64
  }

  private decodeEventValue(value: SorobanRpc.xdr.ScVal): any {
    const scValType = value.switch().name;

    switch (scValType) {
      case 'scvBool':
        return value.b();
      case 'scvU64':
        return BigInt(value.u64().toString());
      case 'scvI64':
        return BigInt(value.i64().toString());
      case 'scvU128':
        const u128 = value.u128();
        return BigInt(u128.hi().toString()) << 64n | BigInt(u128.lo().toString());
      case 'scvI128':
        const i128 = value.i128();
        return BigInt(i128.hi().toString()) << 64n | BigInt(i128.lo().toString());
      case 'scvSymbol':
        return value.sym().toString();
      case 'scvString':
        return value.str().toString();
      case 'scvAddress':
        return SorobanRpc.Address.fromScVal(value).toString();
      case 'scvMap': {
        const map = value.map();
        if (!map) return {};
        const result: Record<string, any> = {};
        for (const entry of map) {
          const key = this.decodeScValToString(entry.key());
          const val = this.decodeEventValue(entry.val());
          result[key] = val;
        }
        return result;
      }
      case 'scvVec': {
        const vec = value.vec();
        if (!vec) return [];
        return vec.map(v => this.decodeEventValue(v));
      }
      default:
        return value.toXDR('base64');
    }
  }

  private mapTopicToEventType(topic: string): EventType | null {
    // Map contract event topics to our event types
    switch (topic) {
      case 'transfer':
        return EventType.PAYMENT; // Token transfers
      case 'mint':
        return EventType.CONTRACT;
      case 'burn':
        return EventType.CONTRACT;
      case 'trade':
        return EventType.OFFER;
      case 'stake':
        return EventType.CONTRACT;
      case 'unstake':
        return EventType.CONTRACT;
      case 'reward':
        return EventType.CONTRACT;
      default:
        return EventType.CONTRACT; // Generic contract event
    }
  }

  // Method to simulate events for testing
  async simulatePaymentEvent(
    from: string = 'GAIH3ULLFQ4DGSECF2AR555KZ4KNDGEKN4AFI4SU2M7B43MGK3QJZNSR',
    to: string = 'GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN',
    amount: string = '100',
    assetType: string = 'native',
  ): Promise<StellarEvent> {
    const eventData = {
      id: uuidv4(),
      eventType: EventType.PAYMENT,
      ledgerSequence: this.lastLedgerSequence + 1,
      timestamp: new Date().toISOString(),
      transactionHash: `simulated-${uuidv4()}`,
      sourceAccount: from,
      payload: {
        amount,
        assetType,
        from,
        to,
        simulated: true,
      },
    };

    const savedEvent = await this.eventStorageService.saveEvent(eventData);
    await this.webhookDeliveryService.queueEventForDelivery(savedEvent);

    this.logger.log(`Simulated payment event ${savedEvent.id}`);
    return savedEvent;
  }

  async simulateOfferEvent(
    seller: string = 'GAIH3ULLFQ4DGSECF2AR555KZ4KNDGEKN4AFI4SU2M7B43MGK3QJZNSR',
    sellingAmount: string = '1000',
    buyingAmount: string = '50',
  ): Promise<StellarEvent> {
    const price = (
      parseFloat(buyingAmount) / parseFloat(sellingAmount)
    ).toString();

    const eventData = {
      id: uuidv4(),
      eventType: EventType.OFFER,
      ledgerSequence: this.lastLedgerSequence + 1,
      timestamp: new Date().toISOString(),
      transactionHash: `simulated-${uuidv4()}`,
      sourceAccount: seller,
      payload: {
        offerId: 'simulated-' + Date.now(),
        seller,
        sellingAssetType: 'credit_alphanum4',
        sellingAssetCode: 'USD',
        sellingAssetIssuer:
          'GAIH3ULLFQ4DGSECF2AR555KZ4KNDGEKN4AFI4SU2M7B43MGK3QJZNSR',
        buyingAssetType: 'native',
        amount: sellingAmount,
        price,
        type: 'create',
        simulated: true,
      },
    };

    const savedEvent = await this.eventStorageService.saveEvent(eventData);
    await this.webhookDeliveryService.queueEventForDelivery(savedEvent);

    this.logger.log(`Simulated offer event ${savedEvent.id}`);
    return savedEvent;
  }

  getStatus(): {
    isMonitoring: boolean;
    lastLedgerSequence: number;
    lastEventId: string;
    horizonUrl: string;
    sorobanUrl: string;
  } {
    return {
      isMonitoring: this.isMonitoring,
      lastLedgerSequence: this.lastLedgerSequence,
      lastEventId: this.lastEventId,
      horizonUrl: this.horizonServer.serverURL.toString(),
      sorobanUrl: this.sorobanServer.serverURL.toString(),
    };
  }

  async getLatestLedger(): Promise<number> {
    try {
      const ledger = await this.horizonServer
        .ledgers()
        .order('desc')
        .limit(1)
        .call();
      return ledger.records[0].sequence;
    } catch (error) {
      this.logger.error(
        `Failed to get latest ledger: ${error.message}`,
        error.stack,
      );
      return this.lastLedgerSequence;
    }
  }
}
