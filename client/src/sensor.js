(function(global) {
    const OpenSentinel = {
        endpoints: ['/verify'], // Array of endpoints for high availability
        mouseEvents: [],
        keyEvents: [],
        maxEvents: 50, // Keep it lightweight
        initialized: false,
        enablePoW: false,
        onSuccess: null,
        onFailure: null,

        init: function(config) {
            if (this.initialized) return;
            if (config) {
                if (config.endpoint) {
                    this.endpoints = [config.endpoint];
                } else if (config.endpoints && Array.isArray(config.endpoints)) {
                    this.endpoints = config.endpoints;
                }
                if (config.enablePoW !== undefined) this.enablePoW = config.enablePoW;
                if (config.onSuccess) this.onSuccess = config.onSuccess;
                if (config.onFailure) this.onFailure = config.onFailure;
            }

            this.startListening();
            this.initialized = true;
            console.log("OpenSentinel initialized with endpoints:", this.endpoints);
        },

        startListening: function() {
            document.addEventListener('mousemove', (e) => {
                this.recordMouse(e);
            });
            document.addEventListener('keydown', (e) => {
                this.recordKey(e);
            });
        },

        recordMouse: function(e) {
            if (this.mouseEvents.length >= this.maxEvents) this.mouseEvents.shift();
            this.mouseEvents.push([e.clientX, e.clientY, Date.now()]);
        },

        recordKey: function(e) {
            if (this.keyEvents.length >= this.maxEvents) this.keyEvents.shift();
            // Privacy: Do not record the actual key, just "Key" or general category if strict privacy needed.
            // But prompt says "keystroke dynamics", usually implies timing + code.
            // We will store key code for this PoC but in real prod we might just store timing.
            this.keyEvents.push([e.code, Date.now()]);
        },

        // Lightweight Proof-of-Work to deter trivial script abuse
        generatePoW: async function() {
            if (!this.enablePoW) return null;

            const difficulty = 3; // Number of leading zeros required
            const prefix = Date.now().toString() + Math.random().toString();
            let nonce = 0;
            let hash = "";

            // Simple hash function for client-side PoW
            const simpleHash = async (str) => {
                const msgBuffer = new TextEncoder().encode(str);
                const hashBuffer = await crypto.subtle.digest('SHA-256', msgBuffer);
                const hashArray = Array.from(new Uint8Array(hashBuffer));
                return hashArray.map(b => b.toString(16).padStart(2, '0')).join('');
            };

            while (true) {
                hash = await simpleHash(prefix + nonce);
                if (hash.startsWith('0'.repeat(difficulty))) {
                    return { prefix, nonce, hash };
                }
                nonce++;
                // Add a fail-safe break to prevent UI blocking
                if (nonce > 50000) return null;
            }
        },

        verify: async function() {
            let pow = null;
            if (this.enablePoW) {
                pow = await this.generatePoW();
            }

            // Simple payload obfuscation (Base64) to prevent trivial tampering by script kiddies
            const rawPayload = {
                mouse_events: this.mouseEvents,
                key_events: this.keyEvents,
                user_agent: navigator.userAgent,
                timestamp: Date.now(),
                pow: pow
            };

            const payload = {
                data: btoa(JSON.stringify(rawPayload))
            };

            let lastError = null;

            // Iterate through endpoints for High Availability/Decentralised failover
            for (const ep of this.endpoints) {
                try {
                    const response = await fetch(ep, {
                        method: 'POST',
                        headers: {
                            'Content-Type': 'application/json'
                        },
                        body: JSON.stringify(payload)
                    });

                    if (response.ok) {
                        const result = await response.json();
                        if (result.passed && this.onSuccess) {
                            this.onSuccess(result.token || "verified");
                        } else if (!result.passed && this.onFailure) {
                            this.onFailure(result.error || "bot_detected");
                        }
                        return result;
                    }
                } catch (err) {
                    console.warn(`OpenSentinel: Failed to connect to ${ep}, trying next...`, err);
                    lastError = err;
                }
            }

            console.error("OpenSentinel Verification Error: All endpoints failed.", lastError);
            const errResult = { passed: false, error: "Network failure. All endpoints unreachable." };
            if (this.onFailure) this.onFailure(errResult.error);
            return errResult;
        }
    };

    global.OpenSentinel = OpenSentinel;
})(window);
