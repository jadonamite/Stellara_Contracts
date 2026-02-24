# Stellara i18n PoC Service

This is a small proof-of-concept microservice extracting the `i18n` translation domain from the monolith. It demonstrates an independent service with HTTP CRUD endpoints and containerization.

Quick start (local):

```bash
cd Backend/microservices/i18n-service
npm install
npm start
# server will run on http://localhost:3000
```

Endpoints:
- `GET /translations` - list translations
- `POST /translations` - create translation { key, locale, value }

Docker:

```bash
docker build -t stellara-i18n:0.1 .
docker run -p 3000:3000 stellara-i18n:0.1
```
