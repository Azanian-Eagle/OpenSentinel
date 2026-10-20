const test = require('node:test');
const assert = require('node:assert');
const OpenSentinel = require('../src/sensor.js');

test('OpenSentinel initialization', () => {
    // Re-instantiate or reset
    OpenSentinel.initialized = false;
    OpenSentinel.init({
        endpoint: '/custom-verify',
        enablePoW: true
    });

    assert.strictEqual(OpenSentinel.initialized, true);
    assert.deepStrictEqual(OpenSentinel.endpoints, ['/custom-verify']);
    assert.strictEqual(OpenSentinel.enablePoW, true);
});

test('OpenSentinel mouse and key event recording', () => {
    OpenSentinel.mouseEvents = [];
    OpenSentinel.keyEvents = [];

    OpenSentinel.recordMouse({ clientX: 100, clientY: 200 });
    assert.strictEqual(OpenSentinel.mouseEvents.length, 1);
    assert.strictEqual(OpenSentinel.mouseEvents[0][0], 100);
    assert.strictEqual(OpenSentinel.mouseEvents[0][1], 200);

    OpenSentinel.recordKey({ code: 'KeyA' });
    assert.strictEqual(OpenSentinel.keyEvents.length, 1);
    assert.strictEqual(OpenSentinel.keyEvents[0][0], 'KeyA');
});

test('OpenSentinel Proof of Work generation', async () => {
    OpenSentinel.enablePoW = true;
    const pow = await OpenSentinel.generatePoW();
    assert.notStrictEqual(pow, null);
    assert.ok(pow.hash.startsWith('000'));
    assert.ok(pow.nonce >= 0);
});

test('OpenSentinel payload encryption', async () => {
    const plainText = JSON.stringify({ test: 'payload' });
    const encrypted = await OpenSentinel.encryptPayload(plainText);
    assert.ok(encrypted.iv);
    assert.ok(encrypted.ciphertext);
    assert.strictEqual(typeof encrypted.iv, 'string');
    assert.strictEqual(typeof encrypted.ciphertext, 'string');
});
