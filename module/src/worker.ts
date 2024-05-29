// worker.ts
import { parentPort } from 'worker_threads';

parentPort?.on('message', () => {
    setInterval(() => {
        parentPort?.postMessage('callFunction');
    }, 10);
});
