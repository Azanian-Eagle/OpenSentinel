(function (root, factory) {
    if (typeof define === 'function' && define.amd) {
        // AMD. Register as an anonymous module.
        define([], factory);
    } else if (typeof module === 'object' && module.exports) {
        // Node. Does not work with strict CommonJS, but
        // only CommonJS-like environments that support module.exports,
        // like Node.
        module.exports = factory();
    } else {
        // Browser globals (root is window)
        root.OpenSentinel = factory();
    }
}(typeof self !== 'undefined' ? self : this, function () {
    const OpenSentinel = {
        endpoints: [
            '/verify'
        ], // Array of endpoints for high availability
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
                if (config.endpoints && Array.isArray(config.endpoints) && config.endpoints.length > 0) {
                    this.endpoints = config.endpoints;
                } else if (typeof config.endpoint === 'string' && config.endpoint) {
                    this.endpoints = [config.endpoint];
                }
                if (config.enablePoW !== undefined) this.enablePoW = config.enablePoW;
                if (config.onSuccess) this.onSuccess = config.onSuccess;
                if (config.onFailure) this.onFailure = config.onFailure;
            }

            this.startListening();
            this.initialized = true;
            console.log("OpenSentinel initialized with endpoints:", this.endpoints);
        },

        bindEvents: function() {
            this.startListening();
        },

        startListening: function() {
            if (typeof window !== 'undefined' && window.addEventListener) {
                // Event listener bindings safely guarded for window
            }
            if (typeof document !== 'undefined' && document.addEventListener) {
                document.addEventListener('mousemove', (e) => {
                    this.recordMouse(e);
                });
                document.addEventListener('keydown', (e) => {
                    this.recordKey(e);
                });
            }
        },

        recordMouse: function(e) {
            if (this.mouseEvents.length >= this.maxEvents) this.mouseEvents.shift();
            this.mouseEvents.push([e.clientX, e.clientY, Date.now()]);
        },

        recordKey: function(e) {
            if (this.keyEvents.length >= this.maxEvents) this.keyEvents.shift();
            this.keyEvents.push([e.code, Date.now()]);
        },

        // Lightweight Proof-of-Work to deter trivial script abuse
        generatePoW: async function() {
            if (!this.enablePoW) return null;

            const difficulty = 3; // Number of leading zeros required
            const prefix = Date.now().toString() + Math.random().toString();
            let nonce = 0;
            let hash = "";

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
                if (nonce > 500000) return null;
            }
        },

        // Helper to encrypt payload using short-lived session key or fallback
        encryptPayload: async function(plainText, sessionKeyHex) {
            let rawKey;
            if (sessionKeyHex && typeof sessionKeyHex === 'string' && sessionKeyHex.length === 64) {
                const match = sessionKeyHex.match(/.{1,2}/g);
                if (match) {
                    rawKey = new Uint8Array(match.map(byte => parseInt(byte, 16)));
                }
            }

            if (!rawKey) {
                // Development fallback key
                rawKey = new Uint8Array([
                    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
                    17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32
                ]);
            }

            const key = await crypto.subtle.importKey(
                "raw",
                rawKey,
                { name: "AES-GCM" },
                false,
                ["encrypt"]
            );

            const iv = crypto.getRandomValues(new Uint8Array(12));
            const encoder = new TextEncoder();
            const encodedText = encoder.encode(plainText);

            const cipherBuffer = await crypto.subtle.encrypt(
                {
                    name: "AES-GCM",
                    iv: iv
                },
                key,
                encodedText
            );

            const cipherBytes = new Uint8Array(cipherBuffer);

            const ivBase64 = btoa(String.fromCharCode.apply(null, iv));
            const cipherBase64 = btoa(String.fromCharCode.apply(null, cipherBytes));

            return {
                iv: ivBase64,
                ciphertext: cipherBase64
            };
        },

        getChallenge: async function() {
            for (const ep of this.endpoints) {
                try {
                    const baseUrl = ep.replace(/\/verify$/, '');
                    const res = await fetch(`${baseUrl}/challenge`);
                    if (res.ok) {
                        return await res.json();
                    }
                } catch (e) {
                    console.warn("OpenSentinel: Failed to fetch challenge", e);
                }
            }
            return null;
        },

        // Offline Queueing and Retry logic
        offlineQueue: [],
        isTransmitting: false,

        queuePayload: function(payloadData) {
            this.offlineQueue.push(payloadData);
            this.processQueue();
        },

        processQueue: async function() {
            if (this.isTransmitting || this.offlineQueue.length === 0) return;
            this.isTransmitting = true;

            while (this.offlineQueue.length > 0) {
                const payloadData = this.offlineQueue[0];
                let success = false;
                let lastError = null;

                const maxRetries = 3;
                let retryCount = 0;
                let backoffTime = 1000;

                while (retryCount < maxRetries && !success) {
                    for (const ep of this.endpoints) {
                        try {
                            const response = await fetch(ep, {
                                method: 'POST',
                                headers: {
                                    'Content-Type': 'application/json'
                                },
                                body: JSON.stringify(payloadData)
                            });

                            if (response.ok) {
                                const result = await response.json();
                                if (result.passed && this.onSuccess) {
                                    this.onSuccess(result.token || result.message || "verified", result);
                                } else if (!result.passed && this.onFailure) {
                                    this.onFailure(result.message || result.error || "bot_detected", result);
                                }
                                success = true;
                                break;
                            }
                        } catch (err) {
                            console.warn(`OpenSentinel: Failed to connect to ${ep}, retrying...`, err);
                            lastError = err;
                        }
                    }

                    if (!success) {
                        retryCount++;
                        if (retryCount < maxRetries) {
                            console.log(`OpenSentinel: Network unstable. Retrying in ${backoffTime}ms...`);
                            if (typeof window !== 'undefined') {
                                window.dispatchEvent(new CustomEvent('opensentinel-network-unstable', { detail: { retryCount, backoffTime } }));
                            }
                            await new Promise(resolve => setTimeout(resolve, backoffTime));
                            backoffTime *= 2;
                        }
                    }
                }

                if (success) {
                    this.offlineQueue.shift();
                } else {
                    console.error("OpenSentinel Verification Error: All endpoints failed after retries.", lastError);
                    const errResult = { passed: false, error: "Network failure. All endpoints unreachable." };
                    if (this.onFailure) this.onFailure(errResult.error);
                    break;
                }
            }

            this.isTransmitting = false;
        },

        verify: async function() {
            let pow = null;
            if (this.enablePoW) {
                pow = await this.generatePoW();
            }

            let challenge = null;
            try {
                challenge = await this.getChallenge();
            } catch (e) {}

            const rawPayload = {
                mouse_events: this.mouseEvents,
                key_events: this.keyEvents,
                user_agent: typeof navigator !== 'undefined' ? navigator.userAgent : 'Node.js/SSR',
                screen_width: (typeof window !== 'undefined' && window.innerWidth) || 0,
                timestamp: Date.now(),
                pow: pow,
                challenge_id: challenge ? challenge.challenge_id : null
            };

            const sessionKey = challenge && challenge.session_key ? challenge.session_key : null;
            const encryptedData = await this.encryptPayload(JSON.stringify(rawPayload), sessionKey);

            const payload = {
                data: encryptedData.ciphertext,
                iv: encryptedData.iv,
                challenge_id: challenge ? challenge.challenge_id : null
            };

            return new Promise((resolve, reject) => {
                const oldSuccess = this.onSuccess;
                const oldFailure = this.onFailure;

                this.onSuccess = (token, result) => {
                    if (oldSuccess) oldSuccess(token, result);
                    resolve(result || { passed: true, score: 1.0, token: token, message: token || "Verified" });
                };

                this.onFailure = (error, result) => {
                    if (oldFailure) oldFailure(error, result);
                    resolve(result || { passed: false, score: 0.0, message: error || "Verification failed" });
                };

                this.queuePayload(payload);
            });
        }
    };

    return OpenSentinel;
}));
