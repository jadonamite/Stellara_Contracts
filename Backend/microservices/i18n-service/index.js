const express = require('express');
const bodyParser = require('body-parser');
const cors = require('cors');
const { v4: uuidv4 } = require('uuid');

const app = express();
app.use(cors());
app.use(bodyParser.json());

// In-memory store for PoC
const translations = [];

// GET /translations - list all translations
app.get('/translations', (req, res) => {
  res.json(translations);
});

// POST /translations - create translation entry
// body: { key: string, locale: string, value: string }
app.post('/translations', (req, res) => {
  const { key, locale, value } = req.body || {};
  if (!key || !locale || !value) {
    return res.status(400).json({ error: 'key, locale and value are required' });
  }

  const entry = { id: uuidv4(), key, locale, value, createdAt: new Date().toISOString() };
  translations.push(entry);
  res.status(201).json(entry);
});

// Health
app.get('/health', (req, res) => res.json({ status: 'ok' }));

const port = process.env.PORT || 3000;
app.listen(port, () => console.log(`i18n PoC listening on ${port}`));
