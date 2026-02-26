import { Test, TestingModule } from '@nestjs/testing';
import { INestApplication, ValidationPipe } from '@nestjs/common';
import request from 'supertest';
import { TypeOrmModule } from '@nestjs/typeorm';
import { ConfigModule } from '@nestjs/config';
import { AuthModule } from './auth.module';
import { RedisService } from '../redis/redis.service';
import { Keypair } from '@stellar/stellar-sdk';
import * as nacl from 'tweetnacl';

function signMessage(message: string, keypair: Keypair): string {
  const messageBytes = Buffer.from(message, 'utf-8');
  const seed = keypair.rawSecretKey();
  const naclKeypair = nacl.sign.keyPair.fromSeed(seed);
  const signature = nacl.sign.detached(messageBytes, naclKeypair.secretKey);
  return Buffer.from(signature).toString('base64');
}

describe('Auth Integration Tests (e2e)', () => {
  let app: INestApplication;
  let testKeypair: Keypair;
  let testKeypair2: Keypair;
  let incrCounter = 0;
  let evalCounter = 0;

  beforeAll(async () => {
    incrCounter = 0;
    evalCounter = 0;
    testKeypair = Keypair.random();
    testKeypair2 = Keypair.random();

    const mockRedisService = {
      client: {
        get: jest.fn().mockResolvedValue(null),
        set: jest.fn().mockResolvedValue('OK'),
        incr: jest.fn().mockImplementation(() => Promise.resolve(++incrCounter)),
        expire: jest.fn().mockResolvedValue(1),
        eval: jest.fn().mockImplementation(() => Promise.resolve([++evalCounter, 60])),
        del: jest.fn().mockResolvedValue(1),
        keys: jest.fn().mockResolvedValue([]),
        connect: jest.fn().mockResolvedValue(undefined),
        quit: jest.fn().mockResolvedValue(undefined),
      },
    };

    const moduleFixture: TestingModule = await Test.createTestingModule({
      imports: [
        ConfigModule.forRoot({
          isGlobal: true,
          envFilePath: '.env.test',
        }),
        TypeOrmModule.forRoot({
          type: 'sqlite',
          database: ':memory:',
          entities: [__dirname + '/../**/*.entity{.ts,.js}'],
          synchronize: true,
        }),
        AuthModule,
      ],
    })
      .overrideProvider(RedisService)
      .useValue(mockRedisService)
      .compile();

    app = moduleFixture.createNestApplication();
    app.useGlobalPipes(
      new ValidationPipe({
        whitelist: true,
        forbidNonWhitelisted: true,
        transform: true,
      }),
    );
    await app.init();
  }, 30000);

  afterAll(async () => {
    if (app) {
      await app.close();
    }
  });

  describe('Successful Login Flow', () => {
    let accessToken: string;
    let refreshToken: string;
    let nonce: string;

    it('should request a nonce', async () => {
      const response = await request(app.getHttpServer())
        .post('/auth/nonce')
        .send({ publicKey: testKeypair.publicKey() })
        .expect(201);

      expect(response.body).toHaveProperty('nonce');
      expect(response.body).toHaveProperty('expiresAt');
      expect(response.body).toHaveProperty('message');
      nonce = response.body.nonce;
    });

    it('should login with valid signature', async () => {
      const message = `Sign this message to authenticate with Stellara: ${nonce}`;
      const signatureBase64 = signMessage(message, testKeypair);

      const response = await request(app.getHttpServer())
        .post('/auth/wallet/login')
        .send({
          publicKey: testKeypair.publicKey(),
          signature: signatureBase64,
          nonce,
        })
        .expect(200);

      expect(response.body).toHaveProperty('accessToken');
      expect(response.body).toHaveProperty('refreshToken');
      expect(response.body).toHaveProperty('user');
      expect(response.body.user).toHaveProperty('id');

      accessToken = response.body.accessToken;
      refreshToken = response.body.refreshToken;
    });

    it('should access protected endpoint with access token', async () => {
      const response = await request(app.getHttpServer())
        .get('/auth/me')
        .set('Authorization', `Bearer ${accessToken}`)
        .expect(200);

      expect(response.body).toHaveProperty('id');
      expect(response.body.wallets).toBeInstanceOf(Array);
    });

    it('should refresh access token', async () => {
      await new Promise((r) => setTimeout(r, 1100));
      const response = await request(app.getHttpServer())
        .post('/auth/refresh')
        .send({ refreshToken })
        .expect(200);

      expect(response.body).toHaveProperty('accessToken');
      expect(response.body).toHaveProperty('refreshToken');

      expect(response.body.accessToken).not.toBe(accessToken);
      expect(response.body.refreshToken).not.toBe(refreshToken);
    });

    it('should logout successfully', async () => {
      await request(app.getHttpServer())
        .post('/auth/logout')
        .set('Authorization', `Bearer ${accessToken}`)
        .expect(200);
    });
  });

  describe('Replay Attack Prevention', () => {
    it('should reject reused nonce', async () => {
      incrCounter = 0;
      evalCounter = 0;

      const nonceResponse = await request(app.getHttpServer())
        .post('/auth/nonce')
        .send({ publicKey: testKeypair.publicKey() });

      const nonce = nonceResponse.body.nonce;
      const message = `Sign this message to authenticate with Stellara: ${nonce}`;
      const signatureBase64 = signMessage(message, testKeypair);

      await request(app.getHttpServer())
        .post('/auth/wallet/login')
        .send({
          publicKey: testKeypair.publicKey(),
          signature: signatureBase64,
          nonce,
        })
        .expect(200);

      await request(app.getHttpServer())
        .post('/auth/wallet/login')
        .send({
          publicKey: testKeypair.publicKey(),
          signature: signatureBase64,
          nonce,
        })
        .expect(401);
    });
  });

  describe('Invalid Signature', () => {
    it('should reject invalid signature', async () => {
      incrCounter = 0;
      evalCounter = 0;

      const nonceResponse = await request(app.getHttpServer())
        .post('/auth/nonce')
        .send({ publicKey: testKeypair.publicKey() });

      const nonce = nonceResponse.body.nonce;
      const invalidSignature = 'invalid-signature-base64';

      await request(app.getHttpServer())
        .post('/auth/wallet/login')
        .send({
          publicKey: testKeypair.publicKey(),
          signature: invalidSignature,
          nonce,
        })
        .expect(500);
    });
  });

  describe('API Token Flow', () => {
    let accessToken: string;
    let apiToken: string;
    let apiTokenId: string;

    beforeAll(async () => {
      incrCounter = 0;
      evalCounter = 0;

      const nonceResponse = await request(app.getHttpServer())
        .post('/auth/nonce')
        .send({ publicKey: testKeypair.publicKey() });

      const nonce = nonceResponse.body.nonce;
      const message = `Sign this message to authenticate with Stellara: ${nonce}`;
      const signatureBase64 = signMessage(message, testKeypair);

      const loginResponse = await request(app.getHttpServer())
        .post('/auth/wallet/login')
        .send({
          publicKey: testKeypair.publicKey(),
          signature: signatureBase64,
          nonce,
        });

      accessToken = loginResponse.body.accessToken;
    }, 30000);

    it('should create API token', async () => {
      const response = await request(app.getHttpServer())
        .post('/auth/api-token')
        .set('Authorization', `Bearer ${accessToken}`)
        .send({
          name: 'Test AI Service Token',
          role: 'ai-service',
          expiresInDays: 30,
        })
        .expect(201);

      expect(response.body).toHaveProperty('token');
      expect(response.body).toHaveProperty('id');
      expect(response.body.token).toMatch(/^stl_/);

      apiToken = response.body.token;
      apiTokenId = response.body.id;
    });

    it('should list API tokens', async () => {
      const response = await request(app.getHttpServer())
        .get('/auth/api-token')
        .set('Authorization', `Bearer ${accessToken}`)
        .expect(200);

      expect(response.body).toBeInstanceOf(Array);
      expect(response.body.length).toBeGreaterThan(0);
    });

    it('should revoke API token', async () => {
      await request(app.getHttpServer())
        .delete(`/auth/api-token/${apiTokenId}`)
        .set('Authorization', `Bearer ${accessToken}`)
        .expect(200);
    });
  });

  describe('Wallet Binding', () => {
    let accessToken: string;
    let userId: string;

    beforeAll(async () => {
      incrCounter = 0;
      evalCounter = 0;

      const nonceResponse = await request(app.getHttpServer())
        .post('/auth/nonce')
        .send({ publicKey: testKeypair.publicKey() });

      const nonce = nonceResponse.body.nonce;
      const message = `Sign this message to authenticate with Stellara: ${nonce}`;
      const signatureBase64 = signMessage(message, testKeypair);

      const loginResponse = await request(app.getHttpServer())
        .post('/auth/wallet/login')
        .send({
          publicKey: testKeypair.publicKey(),
          signature: signatureBase64,
          nonce,
        });

      accessToken = loginResponse.body.accessToken;
      userId = loginResponse.body.user.id;
    }, 30000);

    it('should bind additional wallet', async () => {
      const nonceResponse = await request(app.getHttpServer())
        .post('/auth/nonce')
        .send({ publicKey: testKeypair2.publicKey() });

      const nonce = nonceResponse.body.nonce;
      const message = `Sign this message to authenticate with Stellara: ${nonce}`;
      const signatureBase64 = signMessage(message, testKeypair2);

      await request(app.getHttpServer())
        .post('/auth/wallet/bind')
        .set('Authorization', `Bearer ${accessToken}`)
        .send({
          publicKey: testKeypair2.publicKey(),
          signature: signatureBase64,
          nonce,
        })
        .expect(201);
    });

  it('should login with second wallet and access same account', async () => {
      incrCounter = 0;
      evalCounter = 0;
      const nonceResponse = await request(app.getHttpServer())
        .post('/auth/nonce')
        .send({ publicKey: testKeypair2.publicKey() });

      const nonce = nonceResponse.body.nonce;
      const message = `Sign this message to authenticate with Stellara: ${nonce}`;
      const signatureBase64 = signMessage(message, testKeypair2);

      const loginResponse = await request(app.getHttpServer())
        .post('/auth/wallet/login')
        .send({
          publicKey: testKeypair2.publicKey(),
          signature: signatureBase64,
          nonce,
        })
        .expect(200);

      expect(loginResponse.body.user.id).toBe(userId);
    });
  });

  describe('Rate Limiting', () => {
    it('should enforce rate limits on nonce endpoint', async () => {
      incrCounter = 0;
      evalCounter = 0;

      const publicKey = Keypair.random().publicKey();

      const requests: Array<Promise<any>> = [];
      for (let i = 0; i < 6; i++) {
        requests.push(
          request(app.getHttpServer()).post('/auth/nonce').send({ publicKey }),
        );
      }

      const responses = await Promise.all(requests);
      const tooManyRequests = responses.filter((r) => r.status === 429);

      expect(tooManyRequests.length).toBeGreaterThan(0);
    });
  });
});