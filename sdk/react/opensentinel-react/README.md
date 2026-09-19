# @azanian-eagle/opensentinel-react

[![npm version](https://img.shields.io/npm/v/@azanian-eagle/opensentinel-react.svg)](https://www.npmjs.com/package/@azanian-eagle/opensentinel-react)

Official React SDK and hook library for **OpenSentinel**, engineered by **Azanian Eagle** to provide seamless, non-invasive CAPTCHA-free human verification and bot detection for modern React applications.

## About OpenSentinel & Azanian Eagle

Engineered by [Azanian Eagle](https://github.com/Azanian-Eagle), OpenSentinel is built on the core principle of **digital sovereignty** and **user privacy**. Legacy CAPTCHA systems force users to solve visual puzzles, exploiting user attention and collecting invasive browser signatures to train commercial third-party models.

OpenSentinel replaces intrusive visual challenges with frictionless client-side telemetry analysis. By evaluating abstract mathematical parameters—such as mouse movement curvature, speed variance, and keystroke flight dynamics—OpenSentinel accurately distinguishes real human users from automated script bots without collecting personally identifiable information (PII).

### Key Features
- **Zero Visual Friction:** Eliminates image selection puzzles and visual challenges entirely.
- **Declarative React Integration:** Clean `useOpenSentinel` custom React hook designed for modern functional components.
- **Privacy-First Architecture:** Zero tracking cookies, zero fingerprinting, and full compliance with South Africa's **Protection of Personal Information Act (POPIA)**, GDPR, and CCPA.
- **Client AES-256-GCM Encryption:** Sensor payload data is encrypted directly on the client side using AES-GCM before network transmission.
- **Cryptographic Proof-of-Work (PoW):** Optional SHA-256 client-side PoW computation deters automated script attacks and high-volume DDoS attempts.
- **First-Class TypeScript Support:** Full TypeScript type definitions included out of the box.

---

## Installation

Install via npm:

```bash
npm install @azanian-eagle/opensentinel-react
```

Or via yarn / pnpm:

```bash
yarn add @azanian-eagle/opensentinel-react
# or
pnpm add @azanian-eagle/opensentinel-react
```

---

## Implementation & Usage Guide

### 1. Basic Component Integration

Import `useOpenSentinel` in your React functional component to handle human verification during form submissions or sensitive actions:

```tsx
import React, { useState } from 'react';
import { useOpenSentinel } from '@azanian-eagle/opensentinel-react';

export const LoginForm: React.FC = () => {
  const [status, setStatus] = useState<string>('');
  const [isSubmitting, setIsSubmitting] = useState<boolean>(false);

  const { verify } = useOpenSentinel({
    endpoints: ['https://api.yourdomain.com/verify'],
    enablePoW: true,
    onSuccess: (token: string) => {
      setStatus('Verification successful! Token: ' + token);
      setIsSubmitting(false);
      // Submit form or navigate
    },
    onFailure: (error: string) => {
      setStatus('Verification failed: ' + error);
      setIsSubmitting(false);
    },
  });

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setIsSubmitting(true);
    setStatus('Evaluating humanity...');

    // Trigger OpenSentinel verification
    await verify();
  };

  return (
    <form onSubmit={handleSubmit} style={{ maxWidth: '400px', margin: '0 auto' }}>
      <h2>Secure Sign In</h2>
      <input type="email" placeholder="Email address" required />
      <input type="password" placeholder="Password" required />

      <button type="submit" disabled={isSubmitting}>
        {isSubmitting ? 'Verifying...' : 'Sign In'}
      </button>

      {status && <p style={{ marginTop: '1rem' }}>{status}</p>}
    </form>
  );
};
```

### 2. Configuration Options (`OpenSentinelConfig`)

The `useOpenSentinel` hook accepts an `OpenSentinelConfig` object:

| Property | Type | Required | Description |
|---|---|---|---|
| `endpoint` | `string` | Optional | Single backend server endpoint URL (e.g. `'https://api.yourdomain.com/verify'`). |
| `endpoints` | `string[]` | Optional | Array of backend endpoints for high-availability fail-over. |
| `enablePoW` | `boolean` | Optional | Set to `true` to enable client-side SHA-256 Proof-of-Work challenge to deter automated bot fleets. |
| `scriptUrl` | `string` | Optional | Explicit URL to load `sensor.js`. Defaults to pulling from the first endpoint in `endpoints` or `endpoint`. |
| `onSuccess` | `(token: string) => void` | Optional | Callback triggered when human verification passes successfully. |
| `onFailure` | `(error: string) => void` | Optional | Callback triggered when verification fails or a bot is detected. |

### 3. Server-Side Rendering (Next.js / SSR) Compatibility

`@azanian-eagle/opensentinel-react` is fully SSR-safe. It validates `typeof window` and `document` before accessing browser APIs, preventing hydration mismatches and build-time errors in frameworks like Next.js (Pages and App Router) or Remix.

In Next.js App Router, ensure you place `'use client';` at the top of components utilizing `useOpenSentinel`:

```tsx
'use client';

import React from 'react';
import { useOpenSentinel } from '@azanian-eagle/opensentinel-react';

export default function ContactSection() {
  const { verify } = useOpenSentinel({
    endpoint: 'https://api.yourdomain.com/verify',
  });

  return (
    <button onClick={verify}>Send Message</button>
  );
}
```

---

## TypeScript Definitions

```typescript
export interface OpenSentinelConfig {
    endpoint?: string;
    endpoints?: string[];
    enablePoW?: boolean;
    onSuccess?: (token: string) => void;
    onFailure?: (error: string) => void;
    scriptUrl?: string;
}

export function useOpenSentinel(config: OpenSentinelConfig): {
    verify: () => Promise<void>;
};
```

---

## Regulatory Compliance & Privacy

OpenSentinel is engineered by **Azanian Eagle** with zero compromises on privacy:
- **POPIA (South Africa):** Complies with lawful processing principles by completely avoiding personal information collection.
- **GDPR & CCPA:** Fully compliant due to zero cross-site tracking, zero browser fingerprinting, and local processing of telemetry dynamics.

---

## Licence

Distributed under the **MIT Licence**. Engineered with pride by **Azanian Eagle**.
