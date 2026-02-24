# Stellara Indexing Infrastructure Guide

## Overview

This guide provides comprehensive recommendations for building off-chain indexing infrastructure to process Stellara contract events efficiently and reliably.

## Architecture Overview

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Blockchain    │───▶│   Event Stream   │───▶│  Event Parser   │───▶│   Database      │
│   (Stellar)    │    │   Collector     │    │   & Normalizer │    │   & Analytics  │
└─────────────────┘    └──────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │                       │
         ▼                       ▼                       ▼                       ▼
    Real-time events        Event filtering        Standardization        Queryable data
    via WebSocket          by contract/type       to schema v1.0        via REST/GraphQL
```

## Core Components

### 1. Event Stream Collector

**Purpose**: Collect raw events from the Stellar blockchain

**Technology Options**:
- **Stellar SDK** (JavaScript/TypeScript)
- **Horizon API** with streaming endpoints
- **Stellar Core** direct connection for high-throughput

**Implementation Example**:

```javascript
const { Server, Horizon } = require('stellar-sdk');

const horizon = new Horizon.Server('https://horizon.stellar.org');
const server = new Server('https://horizon.stellar.org');

// Stream events from specific contracts
const contracts = [
    'GD5K2A7Y6WZQF2DJFQFQ2YQYQ2Q2Q2Q2Q2Q2Q2Q2Q2Q2Q2Q2Q', // Token contract
    'GD3K2A7Y6WZQF2DJFQFQ2YQYQ2Q2Q2Q2Q2Q2Q2Q2Q2Q2Q2Q2Q', // Staking contract
];

contracts.forEach(contractId => {
    server.transactions()
        .forAccount(contractId)
        .stream({
            onmessage: (transaction) => {
                processTransaction(transaction);
            },
            onerror: (error) => {
                console.error('Stream error:', error);
                // Implement retry logic
            }
        });
});
```

### 2. Event Parser & Normalizer

**Purpose**: Parse raw events and normalize to standard schema

**Key Responsibilities**:
- Extract events from transaction results
- Validate event structure
- Convert to standardized format
- Handle both legacy and new event formats

**Implementation Example**:

```python
from stellar_sdk import Server
from stellar_sdk.xdr import TransactionResult
import json
from typing import Dict, List, Optional

class StellaraEventParser:
    def __init__(self):
        self.event_schemas = self.load_event_schemas()
    
    def parse_transaction(self, transaction_result: TransactionResult) -> List[Dict]:
        """Parse transaction and extract Stellara events"""
        events = []
        
        if not transaction_result.result.results:
            return events
            
        for result in transaction_result.result.results:
            if result.tr and result.tr.type == 0:  # Contract invocation
                contract_events = self.extract_contract_events(result)
                events.extend(contract_events)
        
        return events
    
    def extract_contract_events(self, result) -> List[Dict]:
        """Extract and normalize contract events"""
        events = []
        
        # Check for standardized events (new format)
        if self.is_standardized_event(result):
            events.append(self.parse_standardized_event(result))
        else:
            # Handle legacy events
            legacy_events = self.parse_legacy_events(result)
            events.extend(legacy_events)
        
        return events
    
    def is_standardized_event(self, result) -> bool:
        """Check if event follows new standardized format"""
        # Check for "stellara_event" topic
        return hasattr(result, 'event') and result.event.topic == 'stellara_event'
    
    def parse_standardized_event(self, result) -> Dict:
        """Parse standardized event to common format"""
        event_data = result.event.body.v0
        
        return {
            'event_type': event_data.topics[1],
            'contract_address': event_data.topics[0],
            'user_address': event_data.topics[1] if event_data.topics[1] else None,
            'data': event_data.data[0],
            'metadata': event_data.data[1],
            'timestamp': event_data.data[2],
            'version': event_data.data[3],
            'transaction_hash': result.transaction_hash,
            'ledger_sequence': result.ledger_sequence,
        }
```

### 3. Database Schema

**Recommended Database**: PostgreSQL with JSONB support

**Core Tables**:

```sql
-- Main events table
CREATE TABLE stellara_events (
    id BIGSERIAL PRIMARY KEY,
    transaction_hash VARCHAR(64) NOT NULL UNIQUE,
    ledger_sequence BIGINT NOT NULL,
    contract_address VARCHAR(66) NOT NULL,
    user_address VARCHAR(66),
    event_type VARCHAR(50) NOT NULL,
    event_version INTEGER NOT NULL DEFAULT 1,
    timestamp BIGINT NOT NULL,
    data JSONB NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX idx_events_ledger ON stellara_events(ledger_sequence);
CREATE INDEX idx_events_contract ON stellara_events(contract_address);
CREATE INDEX idx_events_user ON stellara_events(user_address);
CREATE INDEX idx_events_type ON stellara_events(event_type);
CREATE INDEX idx_events_timestamp ON stellara_events(timestamp);
CREATE INDEX idx_events_metadata ON stellara_events USING GIN(metadata);
CREATE INDEX idx_events_data ON stellara_events USING GIN(data);

-- Contract registry
CREATE TABLE contracts (
    address VARCHAR(66) PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    type VARCHAR(50) NOT NULL, -- token, staking, governance, trading
    version VARCHAR(20) NOT NULL,
    deployed_at TIMESTAMP WITH TIME ZONE NOT NULL,
    metadata JSONB DEFAULT '{}'
);

-- Event statistics
CREATE TABLE event_statistics (
    id BIGSERIAL PRIMARY KEY,
    contract_address VARCHAR(66) NOT NULL,
    event_type VARCHAR(50) NOT NULL,
    date DATE NOT NULL,
    event_count BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(contract_address, event_type, date)
);

-- Materialized views for common queries
CREATE MATERIALIZED VIEW daily_token_transfers AS
SELECT 
    DATE(to_timestamp(timestamp/1000)) as date,
    contract_address,
    SUM((data->>'amount')::BIGINT) as total_amount,
    COUNT(*) as transfer_count
FROM stellara_events 
WHERE event_type = 'transfer'
GROUP BY DATE(to_timestamp(timestamp/1000)), contract_address;

CREATE INDEX idx_daily_transfers_date ON daily_token_transfers(date);
CREATE INDEX idx_daily_transfers_contract ON daily_token_transfers(contract_address);
```

### 4. API Layer

**REST API Endpoints**:

```javascript
// Express.js API example
const express = require('express');
const { Pool } = require('pg');

const app = express();
const pool = new Pool({ /* db config */ });

// Get events by contract
app.get('/api/events/contract/:address', async (req, res) => {
    const { address } = req.params;
    const { type, from, to, limit = 100, offset = 0 } = req.query;
    
    let query = `
        SELECT * FROM stellara_events 
        WHERE contract_address = $1
    `;
    const params = [address];
    
    if (type) {
        query += ` AND event_type = $${params.length + 1}`;
        params.push(type);
    }
    
    query += ` ORDER BY timestamp DESC LIMIT $${params.length + 1} OFFSET $${params.length + 2}`;
    params.push(limit, offset);
    
    const result = await pool.query(query, params);
    res.json(result.rows);
});

// Get user activity
app.get('/api/users/:address/events', async (req, res) => {
    const { address } = req.params;
    const { type, limit = 50 } = req.query;
    
    const query = `
        SELECT * FROM stellara_events 
        WHERE user_address = $1
        ${type ? 'AND event_type = $2' : ''}
        ORDER BY timestamp DESC 
        LIMIT $${type ? '3' : '2'}
    `;
    
    const params = type ? [address, type, limit] : [address, limit];
    const result = await pool.query(query, params);
    res.json(result.rows);
});

// Get token analytics
app.get('/api/analytics/tokens/:address', async (req, res) => {
    const { address } = req.params;
    const { period = '7d' } = req.query;
    
    const query = `
        SELECT 
            DATE(to_timestamp(timestamp/1000)) as date,
            SUM((data->>'amount')::BIGINT) as volume,
            COUNT(*) as transaction_count
        FROM stellara_events 
        WHERE contract_address = $1 
        AND event_type = 'transfer'
        AND timestamp >= $2
        GROUP BY DATE(to_timestamp(timestamp/1000))
        ORDER BY date DESC
    `;
    
    const since = getTimestampForPeriod(period);
    const result = await pool.query(query, [address, since]);
    res.json(result.rows);
});
```

## Deployment Architecture

### Production Setup

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Load Balancer │───▶│   API Gateway   │───▶│   API Servers   │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                │                       │
                                ▼                       ▼
                       ┌─────────────────┐    ┌─────────────────┐
                       │ Event Stream    │    │   Database      │
                       │ Processors     │    │   Cluster      │
                       └─────────────────┘    └─────────────────┘
                                │                       │
                                ▼                       ▼
                       ┌─────────────────┐    ┌─────────────────┐
                       │   Redis Cache   │    │   Object Store  │
                       └─────────────────┘    └─────────────────┘
```

### Infrastructure Components

**1. Event Stream Processors**
- **Kubernetes Deployment**: Scalable pod management
- **Horizontal Scaling**: Auto-scale based on event volume
- **Fault Tolerance**: Multiple replicas with failover

**2. Database**
- **PostgreSQL Cluster**: Primary-replica setup
- **Connection Pooling**: PgBouncer for connection management
- **Backup Strategy**: Daily backups with point-in-time recovery

**3. Caching Layer**
- **Redis**: Hot event data and API responses
- **TTL Management**: 5-minute cache for recent events
- **Cache Invalidation**: Event-driven cache updates

**4. Monitoring & Observability**

```yaml
# Prometheus monitoring configuration
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'stellara-indexer'
    static_configs:
      - targets: ['indexer-api:3000']
    metrics_path: '/metrics'
    scrape_interval: 5s

  - job_name: 'postgresql'
    static_configs:
      - targets: ['postgres-exporter:9187']

  - job_name: 'redis'
    static_configs:
      - targets: ['redis-exporter:9121']
```

## Performance Optimization

### 1. Database Optimization

**Partitioning Strategy**:
```sql
-- Partition events table by date
CREATE TABLE stellara_events_y2024m01 PARTITION OF stellara_events
FOR VALUES FROM ('2024-01-01') TO ('2024-02-01');

-- Auto-partitioning function
CREATE OR REPLACE FUNCTION create_monthly_partition()
RETURNS void AS $$
DECLARE
    start_date date;
    end_date date;
    partition_name text;
BEGIN
    start_date := date_trunc('month', CURRENT_DATE);
    end_date := start_date + interval '1 month';
    partition_name := 'stellara_events_y' || to_char(start_date, 'YYYY') || 'm' || to_char(start_date, 'MM');
    
    EXECUTE format('CREATE TABLE IF NOT EXISTS %I PARTITION OF stellara_events FOR VALUES FROM (%L) TO (%L)',
                   partition_name, start_date, end_date);
END;
$$ LANGUAGE plpgsql;
```

**Query Optimization**:
```sql
-- Efficient pagination using cursor-based pagination
SELECT * FROM stellara_events 
WHERE contract_address = $1 
AND timestamp > $2  -- Cursor from last request
ORDER BY timestamp ASC 
LIMIT 100;

-- Precomputed aggregates for dashboard
CREATE MATERIALIZED VIEW contract_daily_stats AS
SELECT 
    contract_address,
    DATE(to_timestamp(timestamp/1000)) as date,
    event_type,
    COUNT(*) as event_count,
    SUM((data->>'amount')::BIGINT) FILTER (WHERE event_type = 'transfer') as total_volume
FROM stellara_events 
GROUP BY contract_address, DATE(to_timestamp(timestamp/1000)), event_type;
```

### 2. Event Processing Optimization

**Batch Processing**:
```python
class BatchEventProcessor:
    def __init__(self, batch_size=1000, flush_interval=5):
        self.batch_size = batch_size
        self.flush_interval = flush_interval
        self.event_buffer = []
        self.last_flush = time.time()
    
    def add_event(self, event):
        self.event_buffer.append(event)
        
        if (len(self.event_buffer) >= self.batch_size or 
            time.time() - self.last_flush >= self.flush_interval):
            self.flush_events()
    
    def flush_events(self):
        if not self.event_buffer:
            return
        
        # Batch insert to database
        self.bulk_insert_events(self.event_buffer)
        self.event_buffer.clear()
        self.last_flush = time.time()
```

**Parallel Processing**:
```python
import asyncio
from concurrent.futures import ThreadPoolExecutor

async def process_events_parallel(events):
    loop = asyncio.get_event_loop()
    
    with ThreadPoolExecutor(max_workers=4) as executor:
        tasks = []
        for event_batch in chunk_events(events, 100):
            task = loop.run_in_executor(
                executor, process_event_batch, event_batch
            )
            tasks.append(task)
        
        results = await asyncio.gather(*tasks)
        return results
```

## Monitoring & Alerting

### Key Metrics

**1. System Health**
- Event processing lag (seconds behind latest ledger)
- Database connection pool utilization
- API response times
- Error rates by component

**2. Business Metrics**
- Events processed per minute
- Unique active contracts
- User activity levels
- Transaction volumes

**3. Alerting Rules**

```yaml
# Prometheus alerting rules
groups:
  - name: stellara-indexer
    rules:
      - alert: EventProcessingLag
        expr: time() - stellar_last_processed_timestamp > 300
        for: 2m
        labels:
          severity: warning
        annotations:
          summary: "Event processing is lagging behind"
          
      - alert: HighErrorRate
        expr: rate(stellara_errors_total[5m]) > 0.1
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "High error rate in event processing"
          
      - alert: DatabaseConnectionsHigh
        expr: pg_stat_activity_count > 80
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Database connection pool nearly exhausted"
```

## Security Considerations

### 1. Data Validation
- Validate event structure before processing
- Sanitize user inputs in API endpoints
- Implement rate limiting per client

### 2. Access Control
```javascript
// API key authentication
const authenticate = (req, res, next) => {
    const apiKey = req.headers['x-api-key'];
    if (!apiKey || !isValidApiKey(apiKey)) {
        return res.status(401).json({ error: 'Unauthorized' });
    }
    next();
};

// Rate limiting
const rateLimit = require('express-rate-limit');
app.use(rateLimit({
    windowMs: 15 * 60 * 1000, // 15 minutes
    max: 1000, // limit each IP to 1000 requests per windowMs
    message: 'Too many requests from this IP'
}));
```

### 3. Data Privacy
- Encrypt sensitive data at rest
- Use TLS for all communications
- Implement audit logging for data access

## Testing Strategy

### 1. Unit Tests
```python
import pytest
from event_parser import StellaraEventParser

class TestEventParser:
    def test_parse_standardized_transfer_event(self):
        # Mock transaction result
        mock_result = create_mock_transfer_event()
        
        parser = StellaraEventParser()
        events = parser.parse_transaction(mock_result)
        
        assert len(events) == 1
        assert events[0]['event_type'] == 'transfer'
        assert events[0]['version'] == 1
        
    def test_parse_legacy_event_compatibility(self):
        # Test backward compatibility
        mock_result = create_mock_legacy_event()
        
        parser = StellaraEventParser()
        events = parser.parse_transaction(mock_result)
        
        assert len(events) == 1
        # Verify legacy event is properly normalized
```

### 2. Integration Tests
```python
import asyncio
import aiohttp

async def test_event_streaming():
    """Test end-to-end event processing"""
    
    # Send test transaction to testnet
    tx_hash = await send_test_transaction()
    
    # Wait for event processing
    await asyncio.sleep(10)
    
    # Verify event appears in API
    async with aiohttp.ClientSession() as session:
        async with session.get(f'http://localhost:3000/api/events/tx/{tx_hash}') as resp:
            events = await resp.json()
            assert len(events) > 0
```

### 3. Load Testing
```python
# Locust load testing example
from locust import HttpUser, task, between

class ApiUser(HttpUser):
    wait_time = between(1, 3)
    
    @task(3)
    def get_events(self):
        self.client.get("/api/events/contract/TEST_CONTRACT")
    
    @task(1)
    def get_user_activity(self):
        self.client.get("/api/users/TEST_USER/events")
```

## Deployment Checklist

### Pre-deployment
- [ ] Database schema migrations tested
- [ ] API endpoints documented and tested
- [ ] Monitoring dashboards configured
- [ ] Alert rules verified
- [ ] Backup procedures tested
- [ ] Security audit completed

### Post-deployment
- [ ] Event processing lag monitored
- [ ] API performance validated
- [ ] Error rates checked
- [ ] User acceptance testing completed

## Maintenance

### Regular Tasks
- **Daily**: Monitor event processing lag, check error rates
- **Weekly**: Review database performance, update statistics
- **Monthly**: Apply security patches, review capacity planning
- **Quarterly**: Performance tuning, architecture review

### Troubleshooting Guide

**Common Issues**:
1. **Events Missing**: Check event stream connectivity, verify contract addresses
2. **High Latency**: Review database queries, check indexing
3. **Memory Issues**: Monitor batch sizes, adjust processing limits
4. **Database Locks**: Review transaction isolation levels

**Debug Tools**:
- Event processing logs with correlation IDs
- Database query performance analysis
- API request tracing
- System resource monitoring

For additional support, see the [Stellara Documentation](https://docs.stellara.network) and create issues in the [GitHub Repository](https://github.com/stellara-network/Stellara_Contracts).
