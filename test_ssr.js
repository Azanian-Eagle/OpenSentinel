const OpenSentinel = require('./client/src/sensor.js');
console.log('OpenSentinel loaded in Node.js:', Object.keys(OpenSentinel));
OpenSentinel.init({ endpoint: 'http://localhost:8080/verify' });
console.log('Initialized properly.');
