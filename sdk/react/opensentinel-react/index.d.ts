export interface OpenSentinelConfig {
    endpoint?: string;
    endpoints?: string[];
    enablePoW?: boolean;
    onSuccess?: (token: string) => void;
    onFailure?: (error: string) => void;
    scriptUrl?: string;
}
declare global {
    interface Window {
        OpenSentinel?: {
            init: (config: Omit<OpenSentinelConfig, 'scriptUrl'>) => void;
            verify: () => Promise<void>;
        };
    }
}
export declare function useOpenSentinel(config: OpenSentinelConfig): {
    verify: () => Promise<void>;
};
//# sourceMappingURL=index.d.ts.map