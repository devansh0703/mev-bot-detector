// services/backend/load-test.js (Corrected for Batching)
const { Kafka } = require('kafkajs');
const { randomBytes } = require('crypto');
const config = require('./config');
const logger = require('./logger');

function createMockTx() {
    return {
        victim_tx_hash: `0x${randomBytes(32).toString('hex')}`,
        attacker: `0x${randomBytes(20).toString('hex')}`,
        frontrun_tx_hash: `0x${randomBytes(32).toString('hex')}`,
        backrun_tx_hash: `0x${randomBytes(32).toString('hex')}`,
        profit_eth: (Math.random() * 2).toFixed(4),
        timestamp: Math.floor(Date.now() / 1000),
    };
}

async function runLoadTest() {
    const kafka = new Kafka({ clientId: 'load-tester', brokers: [config.kafkaBroker] });
    const producer = kafka.producer();

    logger.info("🚀 Starting load test with batching...");
    await producer.connect();

    const totalMessages = 10000;
    const batchSize = 1000; // Send 1,000 messages at a time
    const batchCount = Math.ceil(totalMessages / batchSize);
    let sentCount = 0;

    const startTime = Date.now();

    try {
        for (let i = 0; i < batchCount; i++) {
            const messages = [];
            for (let j = 0; j < batchSize; j++) {
                messages.push({ value: JSON.stringify(createMockTx()) });
            }

            logger.info(`Sending batch ${i + 1} of ${batchCount}...`);
            await producer.send({
                topic: config.kafkaTopic,
                messages: messages,
            });
            sentCount += messages.length;
        }

        const endTime = Date.now();
        const durationSeconds = (endTime - startTime) / 1000;
        const actualTps = (sentCount / durationSeconds).toFixed(2);

        logger.info(`✅ Load test completed successfully.`);
        logger.info(`Sent ${sentCount} messages in ${durationSeconds.toFixed(2)}s.`);
        logger.info(`🚀 Actual TPS: ${actualTps}`);

    } catch (err) {
        logger.error({ err }, "Error during batched load test.");
    } finally {
        await producer.disconnect();
        logger.info("Load test finished.");
    }
}

runLoadTest().catch(err => {
    logger.error({ err }, "Load test failed catastrophically.");
});
