# @azanian-eagle/react

Official React SDK for OpenSentinel non-invasive CAPTCHA alternative.

## Installation

```bash
npm install @azanian-eagle/react
```

## Usage

```tsx
import React from 'react';
import { useOpenSentinel } from '@azanian-eagle/react';

export const VerificationComponent = () => {
  const { verify } = useOpenSentinel({
    endpoints: ['https://api.yourdomain.com/verify'],
    enablePoW: true,
    onSuccess: (token) => {
      console.log('Verification successful:', token);
    },
    onFailure: (err) => {
      console.error('Verification failed:', err);
    },
  });

  return (
    <div>
      <button onClick={verify}>Verify Humanity</button>
    </div>
  );
};
```

## Licence

MIT
