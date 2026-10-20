const test = require('node:test');
const assert = require('node:assert');
const opensentinelReact = require('../index.js');

test('@azanian-eagle/opensentinel-react exports useOpenSentinel function', () => {
    assert.strictEqual(typeof opensentinelReact.useOpenSentinel, 'function');
});

test('useOpenSentinel hook return signature', () => {
    // Basic verification of export structure
    assert.ok(opensentinelReact.hasOwnProperty('useOpenSentinel'));
});
