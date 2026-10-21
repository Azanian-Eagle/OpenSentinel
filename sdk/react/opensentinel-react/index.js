"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.useOpenSentinel = useOpenSentinel;
const react_1 = require("react");
function useOpenSentinel(config) {
    (0, react_1.useEffect)(() => {
        // If the script isn't specified, default to pulling from endpoint or first endpoints entry (stripping any trailing /verify)
        const rawEndpoint = config.endpoint || (config.endpoints && config.endpoints.length > 0 ? config.endpoints[0] : undefined);
        const cleanBase = rawEndpoint ? rawEndpoint.replace(/\/verify\/?$/, '').replace(/\/$/, '') : undefined;
        const scriptUrl = config.scriptUrl || (cleanBase ? `${cleanBase}/src/sensor.js` : '');
        if (!scriptUrl) {
            console.warn("OpenSentinel: No scriptUrl or endpoint provided.");
            return;
        }
        // Avoid adding the script multiple times
        const existingScript = document.querySelector(`script[src="${scriptUrl}"]`);
        if (!existingScript) {
            const script = document.createElement('script');
            script.src = scriptUrl;
            script.async = true;
            script.onload = () => {
                if (window.OpenSentinel) {
                    const initConfig = { ...config };
                    delete initConfig.scriptUrl;
                    window.OpenSentinel.init(initConfig);
                }
            };
            document.body.appendChild(script);
        }
        else {
            // Already loaded, just init if necessary
            if (window.OpenSentinel) {
                const initConfig = { ...config };
                delete initConfig.scriptUrl;
                window.OpenSentinel.init(initConfig);
            }
        }
    }, [JSON.stringify(config)]);
    const verify = (0, react_1.useCallback)(async () => {
        if (window.OpenSentinel) {
            await window.OpenSentinel.verify();
        }
        else {
            console.warn("OpenSentinel not loaded yet");
        }
    }, []);
    return { verify };
}
//# sourceMappingURL=index.js.map