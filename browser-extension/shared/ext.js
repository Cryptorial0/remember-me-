// Firefox exposes promise-based APIs as `browser`; Chrome/Brave expose them as `chrome`.
export const ext = globalThis.browser ?? globalThis.chrome;
