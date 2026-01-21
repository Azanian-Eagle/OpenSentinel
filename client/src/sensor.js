(function(global) {
    const OpenSentinel = {
        endpoint: '/verify',
        mouseEvents: [],
        keyEvents: [],
        maxEvents: 50, // Keep it lightweight
        initialized: false,

        init: function(config) {
            if (this.initialized) return;
            if (config && config.endpoint) this.endpoint = config.endpoint;

            this.startListening();
            this.initialized = true;
            console.log("OpenSentinel initialized.");
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

        verify: async function() {
            const payload = {
                mouse_events: this.mouseEvents,
                key_events: this.keyEvents,
                user_agent: navigator.userAgent
            };

            try {
                const response = await fetch(this.endpoint, {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json'
                    },
                    body: JSON.stringify(payload)
                });
                return await response.json();
            } catch (err) {
                console.error("OpenSentinel Verification Error:", err);
                return { passed: false, error: err.toString() };
            }
        }
    };

    global.OpenSentinel = OpenSentinel;
})(window);
