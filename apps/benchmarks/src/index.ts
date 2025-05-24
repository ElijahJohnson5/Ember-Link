import './index.css';
import { createClient, User } from '@ember-link/core';
import type { WorkerMessage, UserResult } from './worker';

declare global {
  interface Window {
    startBenchmark: () => void;
    IS_PRODUCTION: boolean;
  }

  interface EmberLink {
    Presence: {
      cursor: { x: number; y: number };
    };
  }
}

function updateCursor(user: User, cursorsContainer: HTMLDivElement) {
  const cursor = getCursorOrCreate(user.clientId, cursorsContainer);

  if (user.cursor) {
    cursor.style.transform = `translateX(${user.cursor.x}px) translateY(${user.cursor.y}px)`;
    cursor.style.opacity = '1';
  } else {
    cursor.style.opacity = '0';
  }
}

// Hard coded array of 25 hex colors
const COLORS = [
  '#2E8B57', // Sea Green
  '#FF6347', // Tomato
  '#4682B4', // Steel Blue
  '#DAA520', // Goldenrod
  '#6A5ACD', // Slate Blue
  '#FF69B4', // Hot Pink
  '#20B2AA', // Light Sea Green
  '#DC143C', // Crimson
  '#3CB371', // Medium Sea Green
  '#FF8C00', // Dark Orange
  '#9370DB', // Medium Purple
  '#00CED1', // Dark Turquoise
  '#F08080', // Light Coral
  '#00BFFF', // Deep Sky Blue
  '#7CFC00', // Lawn Green
  '#D2691E', // Chocolate
  '#FF4500', // Orange Red
  '#ADFF2F', // Green Yellow
  '#1E90FF', // Dodger Blue
  '#DB7093', // Pale Violet Red
  '#40E0D0', // Turquoise
  '#BA55D3', // Medium Orchid
  '#FF1493', // Deep Pink
  '#CD5C5C', // Indian Red
  '#32CD32' // Lime Green
];

function getCursorOrCreate(connectionId: string, cursorsContainer: HTMLDivElement): HTMLElement {
  let cursor: HTMLElement | null = document.getElementById(`cursor-${connectionId}`);

  if (cursor == null) {
    cursor = document.getElementById('cursor-template')!.cloneNode(true) as HTMLElement;
    cursor.id = `cursor-${connectionId}`;
    cursor.style.stroke = COLORS[Math.floor(Math.random() * COLORS.length)];
    cursor.style.display = 'block';
    cursorsContainer.appendChild(cursor);
  }

  return cursor;
}

function deleteCursor(user: User) {
  const cursor = document.getElementById(`cursor-${user.clientId}`);
  if (cursor) {
    cursor.parentNode!.removeChild(cursor);
  }
}

// Main benchmark function
async function startBenchmark(): Promise<void> {
  const baseUrlInput = document.getElementById('baseUrl') as HTMLInputElement;
  const userCountInput = document.getElementById('userCount') as HTMLInputElement;
  const workerCountInput = document.getElementById('workerCount') as HTMLInputElement;
  const output = document.getElementById('output') as HTMLPreElement;
  const cursorsContainer = document.getElementById('cursors-container') as HTMLDivElement;

  if (!userCountInput || !output || !baseUrlInput || !cursorsContainer || !workerCountInput) {
    console.error('Required DOM elements not found.');
    alert('Required DOM elements not found.');
    return;
  }

  output.classList.remove('hidden');

  try {
    new URL(baseUrlInput.value);
  } catch (error) {
    output.textContent = 'Please enter a valid URL';
    return;
  }

  const userCount = parseInt(userCountInput.value, 10);
  if (isNaN(userCount) || userCount <= 0) {
    output.textContent = 'Please enter a valid number of users';
    return;
  }

  const workerCount = parseInt(workerCountInput.value, 10);
  if (isNaN(workerCount) || workerCount <= 0) {
    output.textContent = 'Please enter a valid number of workers';
    return;
  }

  output.textContent = `Starting benchmark with ${userCount} virtual users...\n`;
  output.textContent += `Running with ${workerCount} workers...\n`;

  const results: UserResult[] = [];
  const startTime = performance.now();

  const benchmarkClient = createClient({
    baseUrl: baseUrlInput.value
  });

  const workersPromise: Promise<UserResult[]>[] = [];
  const users = [...Array(userCount).keys()];
  const usersPerWorker = Math.floor(userCount / workerCount);

  for (let i = 0; i < workerCount; i++) {
    let usersForWorker =
      users.length - usersPerWorker < usersPerWorker
        ? users.splice(0, users.length)
        : users.splice(0, usersPerWorker);

    let worker = new Worker('worker.js', {});

    workersPromise.push(
      new Promise<UserResult[]>((resolve) => {
        worker.addEventListener('message', (event: MessageEvent<WorkerMessage>) => {
          if (event.data.type === 'success') {
            console.log('Worker exited successfully');
            worker.terminate();
            resolve(event.data.userResults);
          } else if (event.data.type === 'error') {
            console.error(
              `Error connecting to user ${event.data.error.userId}:`,
              event.data.error.error
            );
          }
        });

        worker.postMessage({
          type: 'start',
          baseUrl: baseUrlInput.value,
          users: usersForWorker
        } satisfies WorkerMessage);
      })
    );
  }

  const { channel: benchmarkChannel, leave: benchmarkChannelLeave } =
    benchmarkClient.joinChannel('test');

  benchmarkChannel.events.others.subscribe('join', (user) => {
    updateCursor(user, cursorsContainer);
  });

  benchmarkChannel.events.others.subscribe('update', (user) => {
    updateCursor(user, cursorsContainer);
  });

  benchmarkChannel.events.others.subscribe('leave', (user) => {
    deleteCursor(user);
  });

  const userResults: UserResult[] = (await Promise.allSettled(workersPromise))
    .map((result) => {
      if (result.status === 'fulfilled') {
        return result.value;
      } else if (result.status === 'rejected') {
        console.error(result.reason);
      }

      return null;
    })
    .reduce((result, current) => {
      if (result) {
        return current!.concat(result);
      }

      return current;
    }, [] as UserResult[]) as UserResult[];

  const endTime = performance.now();
  const totalDuration = endTime - startTime;

  benchmarkChannelLeave();

  output.textContent += `Benchmark completed in ${totalDuration.toFixed(2)}ms\n\n`;
  const aggregateUserResults = userResults.reduce(
    (result, { updateEvents, joinEvents }) => {
      result.updateEvents += updateEvents;
      result.joinEvents += joinEvents;
      result.averageUpdateEventsPerSecond += result.updateEvents / (totalDuration / 1000);
      result.averageJoinEventsPerSecond += result.joinEvents / (totalDuration / 1000);
      return result;
    },
    {
      updateEvents: 0,
      joinEvents: 0,
      averageUpdateEventsPerSecond: 0,
      averageJoinEventsPerSecond: 0
    }
  );

  output.textContent += `Average update events per second: ${(aggregateUserResults.updateEvents / (totalDuration / 1000)).toFixed(2)}\n`;
  output.textContent += `Total number of join events: ${aggregateUserResults.joinEvents}\n`;
  output.textContent += `Total number of update events: ${aggregateUserResults.updateEvents}\n\n`;

  cursorsContainer.innerHTML = '';
  document.querySelectorAll('[id^="cursor-]').forEach((cursor) => {
    if (cursor) {
      cursor.parentNode!.removeChild(cursor);
    }
  });
}

window.startBenchmark = startBenchmark;

if (!window.IS_PRODUCTION) {
  new EventSource('/esbuild').addEventListener('change', () => location.reload());
}
