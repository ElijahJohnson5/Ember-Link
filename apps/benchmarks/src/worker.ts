import { createClient, Channel, EmberClient, User } from '@ember-link/core';

interface ConnectionSuccessful {
  client: EmberClient;
  channel: Channel;
  userId: number;
  leave: () => void;
}

interface ConnectionError {
  userId: number;
  error: string;
}

export interface UserResult {
  userId: number;
  duration: number;
  updateEvents: number;
  joinEvents: number;
}

interface WorkerStartMessage {
  type: 'start';
  baseUrl: string;
  users: number[];
}

interface RateMetric {
  type: 'rate';
  name: string;
}

interface CountMetric {
  type: 'count';
  name: string;
  count: number;
}

interface WorkerMetricMessage {
  type: 'metric';
  metric: RateMetric | CountMetric;
}

interface WorkerErrorMessage {
  type: 'error';
  error: ConnectionError;
}

interface WorkerSuccessMessage {
  type: 'success';
  userResults: UserResult[];
}

export type WorkerMessage =
  | WorkerStartMessage
  | WorkerErrorMessage
  | WorkerSuccessMessage
  | WorkerMetricMessage;

async function connectToEmberLink(userId: number, baseUrl: string): Promise<ConnectionSuccessful> {
  return new Promise((resolve, reject) => {
    const client = createClient({
      baseUrl
    });

    const { channel, leave } = client.joinChannel('test');

    const timeoutId = setTimeout(() => {
      leave();
      reject({
        userId,
        error: 'Connection timed out'
      } satisfies ConnectionError);
    }, 10000);

    channel.events.subscribe('status', (status) => {
      if (status === 'connected') {
        clearTimeout(timeoutId);
        resolve({ client, channel, leave, userId });
      }
    });
  });
}

// Simulate a user performing a task
async function simulateUserCursor(connection: ConnectionSuccessful): Promise<UserResult> {
  const start = performance.now();

  let userJoinEvents = 0;
  connection.channel.events.others.subscribe('join', () => {
    userJoinEvents++;
  });
  let userUpdateEvents = 0;
  connection.channel.events.others.subscribe('update', () => {
    userUpdateEvents++;
  });

  // Simulate some work (e.g., calculation or API call)
  for (let i = 0; i < 1000; i++) {
    connection.channel.updatePresence({
      cursor: { x: Math.floor(Math.random() * 400), y: Math.floor(Math.random() * 400) }
    });

    await new Promise((resolve) => setTimeout(resolve, 17));
  }

  const duration = performance.now() - start;

  return {
    userId: connection.userId,
    duration,
    joinEvents: userJoinEvents,
    updateEvents: userUpdateEvents
  };
}

let started = false;

onmessage = async (event: MessageEvent<WorkerMessage>) => {
  if (event.data.type !== 'start' || started) {
    return;
  }

  started = true;

  const connectionPromises: Promise<ConnectionSuccessful>[] = [];
  const { data } = event;

  for (let i = 1; i <= data.users.length; i++) {
    connectionPromises.push(connectToEmberLink(i, data.baseUrl));
  }

  const connectionResults = await Promise.allSettled(connectionPromises);

  const connections: PromiseFulfilledResult<ConnectionSuccessful>[] = connectionResults.filter(
    (result) => {
      if (result.status === 'fulfilled') {
        return true;
      } else if (result.status === 'rejected') {
        const error = result.reason as ConnectionError;
        postMessage({ error, type: 'error' } satisfies WorkerErrorMessage);
        return false;
      }
    }
  ) as PromiseFulfilledResult<ConnectionSuccessful>[];

  const simulateCursorPromises = connections.map((connection) => {
    return simulateUserCursor(connection.value);
  });

  const results = (await Promise.allSettled(simulateCursorPromises))
    .map((simulatedCursorResults) => {
      if (simulatedCursorResults.status === 'fulfilled') {
        return simulatedCursorResults.value;
      } else {
        return null;
      }
    })
    .filter((result): result is UserResult => {
      return result !== null;
    });

  connections.forEach(({ value: { leave } }) => {
    leave();
  });

  postMessage({ type: 'success', userResults: results } satisfies WorkerSuccessMessage);
};
