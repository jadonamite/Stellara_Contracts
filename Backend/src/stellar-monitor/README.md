# Stellar Blockchain Transaction Monitoring and Event Indexing System

This system provides comprehensive monitoring of Stellar blockchain activities, including transactions, smart contract events, and real-time notifications via webhooks and WebSocket connections.

## Features

### ✅ Core Monitoring Capabilities
- **Real-time Transaction Monitoring**: Monitors Stellar Horizon for payments, offers, and account operations
- **Smart Contract Event Indexing**: Polls Soroban RPC for contract events and decodes XDR data
- **Event Storage**: Efficiently stores indexed events in database with advanced querying
- **Webhook Delivery**: Reliable webhook system with retry logic and consumer management
- **WebSocket Streaming**: Real-time event broadcasting to connected clients
- **Event Filtering**: Consumers can subscribe to specific event types, contracts, or accounts

### ✅ Supported Event Types
- **Payment Events**: Native asset and custom asset transfers
- **Offer Events**: Trading offers (create, update, delete)
- **Contract Events**: Smart contract interactions (transfers, mints, burns, stakes, rewards)
- **Account Events**: Account-related operations
- **Trustline Events**: Asset trustline modifications

## Architecture

### Components

1. **StellarEventMonitorService**
   - Monitors Horizon API for blockchain operations
   - Monitors Soroban RPC for contract events
   - Processes and normalizes event data
   - Queues events for delivery

2. **EventStorageService**
   - Stores events in database with indexing
   - Provides advanced querying capabilities
   - Maintains event processing status

3. **WebhookDeliveryService**
   - Manages webhook consumer subscriptions
   - Handles event filtering and delivery
   - Implements retry logic and failure handling
   - Broadcasts events via WebSocket

4. **ConsumerManagementService**
   - Manages webhook consumer registrations
   - Handles consumer health monitoring
   - Auto-suspends failing consumers

5. **WebSocket Gateway**
   - Real-time event streaming
   - Client subscription management
   - Event filtering for connected clients

## API Endpoints

### Consumer Management
```
POST   /api/stellar/consumers          # Register webhook consumer
GET    /api/stellar/consumers          # List consumers
GET    /api/stellar/consumers/:id      # Get consumer details
PUT    /api/stellar/consumers/:id      # Update consumer
DELETE /api/stellar/consumers/:id      # Delete consumer
POST   /api/stellar/consumers/:id/activate    # Activate consumer
POST   /api/stellar/consumers/:id/deactivate  # Deactivate consumer
POST   /api/stellar/consumers/:id/test        # Test webhook delivery
```

### Event Queries
```
GET    /api/stellar/events                    # List events with filtering
GET    /api/stellar/events/:id               # Get specific event
GET    /api/stellar/events/contract/:contractId  # Events for contract
GET    /api/stellar/events/account/:account     # Events for account
GET    /api/stellar/events/transaction/:txHash  # Events for transaction
```

### Monitoring Control
```
GET    /api/stellar/stats              # System statistics
GET    /api/stellar/health             # Health status
POST   /api/stellar/monitor/start      # Start monitoring
POST   /api/stellar/monitor/stop       # Stop monitoring
GET    /api/stellar/monitor/status     # Monitoring status
```

### Simulation (Testing)
```
POST   /api/stellar/simulate/payment    # Simulate payment event
POST   /api/stellar/simulate/offer     # Simulate offer event
```

## Webhook Consumer Configuration

When registering a webhook consumer, you can specify filters:

```json
{
  "name": "My Application",
  "url": "https://myapp.com/webhook",
  "secret": "webhook-secret",
  "eventTypes": ["payment", "contract"],
  "contractIds": ["contract-id-1", "contract-id-2"],
  "accounts": ["account-id-1"],
  "maxRetries": 5,
  "timeoutMs": 5000
}
```

### Event Filtering
- `eventTypes`: Array of event types to receive
- `contractIds`: Array of contract IDs to monitor
- `accounts`: Array of account addresses to monitor

## WebSocket Real-time Streaming

Connect to the WebSocket endpoint and subscribe to events:

```javascript
const socket = io('ws://localhost:3000');

socket.emit('subscribe-events', {
  eventTypes: ['payment', 'contract'],
  contractIds: ['my-contract-id']
});

socket.on('stellar-event', (event) => {
  console.log('Received event:', event);
});
```

## Event Data Structure

All events follow a consistent structure:

```json
{
  "id": "uuid",
  "eventType": "payment|contract|offer|...",
  "ledgerSequence": 12345,
  "timestamp": "2024-01-01T00:00:00.000Z",
  "transactionHash": "hash...",
  "sourceAccount": "account-id",
  "payload": {
    // Event-specific data
  }
}
```

## Configuration

Environment variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `HORIZON_URL` | Stellar Horizon API URL | `https://horizon-testnet.stellar.org` |
| `SOROBAN_RPC_URL` | Soroban RPC URL | `https://soroban-testnet.stellar.org` |
| `STELLAR_MONITOR_ENABLED` | Enable/disable monitoring | `true` |
| `WEBHOOK_TIMEOUT` | Webhook request timeout (ms) | `10000` |
| `WEBHOOK_RETRIES` | Max webhook retry attempts | `5` |

## Database Schema

### stellar_events
- `id`: UUID primary key
- `eventType`: Event type enum
- `ledgerSequence`: Ledger sequence number
- `timestamp`: Event timestamp
- `transactionHash`: Transaction hash
- `sourceAccount`: Source account ID
- `payload`: JSON event data
- `deliveryStatus`: Delivery status
- `deliveredTo`: Array of consumer IDs
- `isProcessed`: Processing status

### webhook_consumers
- `id`: UUID primary key
- `name`: Consumer name
- `url`: Webhook URL
- `secret`: Webhook secret
- `eventTypes`: Filtered event types
- `contractIds`: Filtered contract IDs
- `accounts`: Filtered accounts
- `status`: Consumer status
- `isActive`: Active flag

## Monitoring and Health Checks

### Health Endpoint
```
GET /api/stellar/health
```

Returns system health status including:
- Monitor status
- Consumer statistics
- Delivery queue status
- Overall system health

### Statistics Endpoint
```
GET /api/stellar/stats
```

Returns comprehensive statistics:
- Event counts by type
- Consumer metrics
- Delivery performance
- System status

## Error Handling

### Webhook Delivery Failures
- Automatic retry with exponential backoff
- Configurable max retry attempts
- Consumer suspension on repeated failures
- Detailed error logging

### Monitoring Failures
- Automatic stream restart on connection errors
- Graceful degradation
- Alert generation for critical failures

## Security

### Webhook Authentication
- HMAC-SHA256 signature verification
- Configurable webhook secrets
- Request validation

### Rate Limiting
- Configurable timeout settings
- Consumer-specific limits
- Automatic suspension of abusive consumers

## Testing

Run the test suite:

```bash
npm run test:e2e
```

Tests include:
- Consumer registration and management
- Event simulation and retrieval
- Webhook delivery testing
- System health validation

## Production Deployment

### Scaling Considerations
- Use Redis for distributed webhook queues
- Implement database read replicas for queries
- Consider event partitioning by contract/account
- Monitor WebSocket connection limits

### Reliability
- Implement proper logging and monitoring
- Set up alerts for system health
- Use database backups and replication
- Implement graceful shutdown procedures

### Performance Optimization
- Database indexing on frequently queried fields
- Event batching for bulk operations
- Connection pooling for external APIs
- Caching for frequently accessed data