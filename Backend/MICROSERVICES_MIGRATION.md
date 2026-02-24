# Stellara Backend: Migration to Domain-Driven Microservices

This document describes a production-ready migration strategy for moving the Stellara monolithic backend to a domain-driven microservices architecture. It includes service boundaries, communication patterns, operational requirements, a phased migration roadmap, and a small proof-of-concept extraction for the `i18n` domain.

---

**Scope & Goals**
- **Goals:** independent service scaling, faster deployments, fault isolation, team autonomy, improved observability and resilience.
- **Non-goals:** complete rewrite of business logic; instead perform incremental extraction using the Strangler Fig pattern.

---

**1. Domain & Service Decomposition**

Approach:
- Perform event, code, and data analysis to identify bounded contexts using DDD techniques (Ubiquitous Language, context maps, aggregates).
- Use runtime telemetry and code dependency graphs (call traces, import graph) to detect natural seams.

Candidate services (examples):
- Auth & Identity: user auth, tokens, sessions, permission store.
- Billing & Payments: payment processing, invoices, usage metering.
- Trading / Orders: trading engine, order book, matching, settlement.
- Messaging: real-time websockets/gateways, pubsub for client messaging.
- Notifications: e-mail, SMS, push.
- AI Jobs: long-running model jobs, inference tasks.
- Analytics & Events: event storage, OLAP, metrics pipeline.
- I18N: translations, locale resources, translation CRUD (good PoC candidate).
- Audit & Compliance: audit logs, immutable storage.

For each service define:
- Responsibility: explicit behavioral contract.
- Data ownership: single source-of-truth DB per service.
- Owner: team or role for lifecycle.

Shared modules requiring extraction to platform services:
- Auth (token validation / session management), Logging, Observability, Config, Rate-limiting, Feature flags, Common utilities.

---

**2. Architecture Design**

High-level components:
- API Gateway: ingress, auth pre-validation, routing, rate limits, JWT validation (Kong/Traefik/NGINX + lua plugins or AWS API GW).
- Service Mesh: mTLS, service discovery, telemetry (Linkerd / Istio) if using Kubernetes.
- Event Backbone: Kafka (for high-throughput ordered streams) or NATS/Redis Streams for lower-latency pub-sub.
- Datastores: one database per service (Postgres, Mongo, or other polyglot choices) with explicit ownership.

Communication patterns:
- Synchronous: REST/gRPC for low-latency request/response (gRPC preferred for inter-service binary contracts and performance).
- Asynchronous: event-driven for eventual consistency, decoupling, and high-throughput data flows.

Resilience patterns:
- Circuit Breakers (e.g., via Envoy/Hystrix-style libraries), retries with exponential backoff, timeouts, bulkheading by thread/connection pools.
- Idempotency tokens on async operations.

---

**3. Communication & Integration Strategy**

Protocols:
- External client -> Gateway: HTTPS/REST + JWT OAuth2/OIDC.
- Service -> Service (sync): gRPC (Protobuf) for internal RPCs.
- Async events: Kafka (topic per aggregate) or NATS JetStream for event streaming. Use Avro/Protobuf for schemas.

Contract-first approach:
- Define OpenAPI for external APIs and Protobuf for internal RPC and events.
- Maintain a central contract repo with automated CI that publishes client stubs and validators.

Versioning:
- Semantic versioning for API contracts; prefer additive changes (new endpoints/topics) and versioned endpoints for breaking changes.

Orchestration/Sagas:
- Use choreography for simple long-running flows (events trigger subsequent steps).
- Use orchestrators (Temporal, Cadence, or a dedicated Saga service) for complex distributed transactions requiring compensation.

---

**4. Data & State Migration**

Strategy:
- Start by splitting schemas per bounded context. Create a migration plan per table/entity identifying owner service.
- For live data, use dual-write during transition with an adapter layer or a change-data-capture (CDC) approach (Debezium -> Kafka) to bootstrap new service databases.

Consistency:
- Adopt eventual consistency. Use events as the source-of-truth for cross-service sync.
- When strict consistency required, implement synchronous calls with timeouts and fallbacks.

Options:
- Data duplication: store denormalized copies in consumer services where read performance matters.
- Event sourcing / CQRS: accelerate on aggregates with high auditability; introduce only where benefits outweigh cost.

---

**5. Infrastructure & Platform Enablement**

Kubernetes-based deployment:
- Containerize services with Docker, use Helm charts and namespaces per environment.
- Service discovery via Kubernetes DNS; optionally use Consul for multi-cluster.

Security & config:
- Centralized config via Vault or Kubernetes secrets; implement mTLS between services.
- CI/CD pipelines per service (GitHub Actions / GitLab CI), automated image scans and policy checks.

Platform services:
- Logging, Metrics, Tracing, Identity Provider (OIDC), Secrets, Message Broker, Storage, DB provisioning.

---

**6. Observability & Reliability**

Tracing & logging:
- Use OpenTelemetry for tracing; collect traces in Jaeger/Tempo.
- Centralized logging with Loki/ELK; context-enrich logs with trace IDs and request IDs.

Metrics & SLOs:
- Expose Prometheus metrics from each service.
- Define SLIs (latency, error rate), SLOs and error budgets per service.

Health & auto-heal:
- Liveness and readiness probes in Kubernetes; automated restart and HPA scaling rules.

---

**7. Migration Roadmap & Execution Strategy**

Phasing (example):
1. Platform setup: messaging, observability, API gateway, CI/CD templates.
2. Extract low-risk services (I18N, Audit read-only services) using Strangler Fig.
3. Extract stateful services with CDC (Billing, Orders) and migrate databases.
4. Move critical paths (Auth, Payments) with canary & blue-green deploys.

Deployment strategies:
- Canary releases + traffic shaping with the API Gateway.
- Blue-green for major DB migrations.

Rollback:
- Keep backward-compatible endpoints; maintain dual readers; use feature flags to toggle.

Risks & mitigations:
- Data divergence: mitigate via CDC verification and reconciler jobs.
- Latency added by async flows: monitor and offer sync fallbacks when necessary.

---

**8. Security & Governance**

- Enforce mutual TLS, service-to-service auth with short-lived service tokens.
- API Gateway performs edge validation, rate limits, and WAF controls.
- Governance: templates, onboarding docs, service checklist (metrics, readiness, tracing), CI checks, and a central contract registry.

---

**Deliverables (this repo)**
- This migration plan.
- PoC microservice: `Backend/microservices/i18n-service` (simple HTTP service, Dockerfile, OpenAPI stub).
- Migration checklist and sample Helm/CI snippets (recommended next step).

---

**Proof-of-Concept: `i18n` extraction**
- Purpose: demonstrate extracting a well-contained bounded context with CRUD endpoints, independent DB (in PoC, in-memory), API contract, and containerization.
- Location: `Backend/microservices/i18n-service`.

How to run (local PoC):
1. cd Backend/microservices/i18n-service
2. npm install
3. npm start

This service exposes:
- GET /translations
- POST /translations

---

**Next steps & recommendations**
- Run a targeted code & data dependency analysis to finalize service boundaries.
- Stand up platform infra (k8s cluster, Kafka, observability). Use a sandbox environment for the first extractions.
- Begin PoC validation with load & fault-injection tests.

---

Appendix: contract and event examples, sample OpenAPI and Protobuf schemas, and CDC pattern (Debezium -> Kafka) should be created in a follow-up PR.
