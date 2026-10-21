import { useEffect, useCallback } from 'react';

export interface OpenSentinelConfig {
    endpoint?: string;
    endpoints?: string[];
    enablePoW?: boolean;
    onSuccess?: (token: string) => void;
    onFailure?: (error: string) => void;
    scriptUrl?: string; // e.g. "http://localhost:8080/src/sensor.js"
}

// Ensure typescript knows about the global OpenSentinel object injected by sensor.js
declare global {
    interface Window {
        OpenSentinel?: {
            init: (config: Omit<OpenSentinelConfig, 'scriptUrl'>) => void;
            verify: () => Promise<void>;
        };
    }
}

export function useOpenSentinel(config: OpenSentinelConfig) {
    useEffect(() => {
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
                    delete (initConfig as any).scriptUrl;
                    window.OpenSentinel.init(initConfig);
                }
            };
            document.body.appendChild(script);
        } else {
            // Already loaded, just init if necessary
            if (window.OpenSentinel) {
                 const initConfig = { ...config };
                 delete (initConfig as any).scriptUrl;
                 window.OpenSentinel.init(initConfig);
            }
        }
    }, [JSON.stringify(config)]);

    const verify = useCallback(async () => {
        if (window.OpenSentinel) {
            await window.OpenSentinel.verify();
        } else {
            console.warn("OpenSentinel not loaded yet");
        }
    }, []);

    return { verify };
}
