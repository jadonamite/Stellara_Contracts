import { Test, TestingModule } from '@nestjs/testing';
import { INestApplication } from '@nestjs/common';
import * as request from 'supertest';
import { ConfigModule } from '@nestjs/config';
import { AppModule } from '../src/app.module';

jest.setTimeout(30000);

describe('Stellar Monitor (e2e)', () => {
  let app: INestApplication;

  beforeAll(async () => {
    const moduleRef: TestingModule = await Test.createTestingModule({
      imports: [
        ConfigModule.forRoot({ envFilePath: '.env.test', isGlobal: true }),
        AppModule,
      ],
    }).compile();

    app = moduleRef.createNestApplication();
    await app.init();
  });

  afterAll(async () => {
    if (app) {
      await app.close();
    }
  });

  it('should create and retrieve webhook consumer', async () => {
    const consumerData = {
      name: 'Test Consumer',
      url: 'https://example.com/webhook',
      secret: 'test-secret',
      eventTypes: ['payment', 'contract'],
      contractIds: ['test-contract-id'],
    };

    // Create consumer
    const createResponse = await request(app.getHttpServer())
      .post('/api/stellar/consumers')
      .send(consumerData)
      .expect(201);

    expect(createResponse.body).toBeDefined();
    expect(createResponse.body.id).toBeDefined();

    const consumerId = createResponse.body.id;

    // Retrieve consumer
    const getResponse = await request(app.getHttpServer())
      .get(`/api/stellar/consumers/${consumerId}`)
      .expect(200);

    expect(getResponse.body.id).toBe(consumerId);
    expect(getResponse.body.name).toBe(consumerData.name);
    expect(getResponse.body.eventTypes).toEqual(consumerData.eventTypes);
  });

  it('should simulate and retrieve events', async () => {
    // Simulate a payment event
    const simulateResponse = await request(app.getHttpServer())
      .post('/api/stellar/simulate/payment')
      .send({
        from: 'GAIH3ULLFQ4DGSECF2AR555KZ4KNDGEKN4AFI4SU2M7B43MGK3QJZNSR',
        to: 'GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN',
        amount: '100',
        assetType: 'native',
      })
      .expect(201);

    expect(simulateResponse.body).toBeDefined();
    expect(simulateResponse.body.id).toBeDefined();

    const eventId = simulateResponse.body.id;

    // Retrieve the event
    const getEventResponse = await request(app.getHttpServer())
      .get(`/api/stellar/events/${eventId}`)
      .expect(200);

    expect(getEventResponse.body.id).toBe(eventId);
    expect(getEventResponse.body.eventType).toBe('payment');
    expect(getEventResponse.body.payload.amount).toBe('100');
  });

  it('should return monitoring stats', async () => {
    const statsResponse = await request(app.getHttpServer())
      .get('/api/stellar/stats')
      .expect(200);

    expect(statsResponse.body).toBeDefined();
    expect(statsResponse.body.events).toBeDefined();
    expect(statsResponse.body.consumers).toBeDefined();
    expect(statsResponse.body.delivery).toBeDefined();
    expect(statsResponse.body.monitor).toBeDefined();
  });

  it('should return health status', async () => {
    const healthResponse = await request(app.getHttpServer())
      .get('/api/stellar/health')
      .expect(200);

    expect(healthResponse.body).toBeDefined();
    expect(healthResponse.body.status).toBeDefined();
    expect(['healthy', 'degraded', 'unhealthy']).toContain(healthResponse.body.status);
  });
});