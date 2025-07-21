// services/backend/config.js
require('dotenv').config();

const config = {
  wssUrl: process.env.WSS_URL,
  kafkaBroker: process.env.KAFKA_BROKER,
  redisUrl: process.env.REDIS_URL,
  kafkaTopic: process.env.KAFKA_TOPIC || 'mev-alerts',
  logLevel: process.env.LOG_LEVEL || 'info',
  batchSize: parseInt(process.env.BATCH_SIZE, 10) || 100,
  batchIntervalMs: parseInt(process.env.BATCH_INTERVAL_MS, 10) || 1000,
  uniswapSubgraphUrl: 'https://api.thegraph.com/subgraphs/name/uniswap/uniswap-v2',
};

// Validate essential configuration
if (!config.wssUrl) {
  console.error("FATAL: WSS_URL is not defined in the environment. Please check your .env file.");
  process.exit(1);
}

module.exports = config;
