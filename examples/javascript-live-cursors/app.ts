import { createClient } from '../../packages/core';

const client = createClient({
  baseUrl: 'ws://localhost:9000'
});

const channel = client.joinChannel('test');

const cursorsContainer = document.getElementById('cursors-container')!;
const text = document.getElementById('text')!;

channel.events.subscribe('presence', (presence) => {
  const cursor: { x: number; y: number } | null =
    (presence?.custom?.cursor as { x: number; y: number }) ?? null;

  text.innerHTML = cursor
    ? `${cursor.x} × ${cursor.y}`
    : 'Move your cursor to broadcast its position to other people in the channel.';
});

/**
 * Subscribe to every others presence updates.
 * The callback will be called if you or someone else enters or leaves the channel
 * or when someone presence is updated
 */
channel.events.others.subscribe('join', (user) => {
  console.log('User join: ', user);
  updateCursor(user);
});

channel.events.others.subscribe('update', (user) => {
  console.log('User update: ', user);
  updateCursor(user);
});

channel.events.others.subscribe('leave', (user) => {
  console.log('User leave: ', user);
  deleteCursor(user);
});

channel.events.others.subscribe('reset', () => {
  cursorsContainer.innerHTML = '';
  for (const cursor of document.querySelectorAll('[id^="cursor-]')) {
    if (cursor) {
      cursor.parentNode!.removeChild(cursor);
    }
  }
});

document.addEventListener('pointermove', (event) => {
  channel.updatePresence({
    custom: {
      cursor: { x: Math.round(event.clientX), y: Math.round(event.clientY) }
    }
  });
});

document.addEventListener('pointerleave', (e) => {
  channel.updatePresence({ custom: { cursor: null } });
});

const COLORS = ['#DC2626', '#D97706', '#059669', '#7C3AED', '#DB2777'];

// Update cursor position based on user presence
function updateCursor(user: {
  clientId: string;
  custom: {
    cursor: {
      x: number;
      y: number;
    };
  };
}) {
  const cursor = getCursorOrCreate(user.clientId);

  if (user.custom?.cursor) {
    cursor.style.transform = `translateX(${user.custom.cursor.x}px) translateY(${user.custom.cursor.y}px)`;
    cursor.style.opacity = '1';
  } else {
    cursor.style.opacity = '0';
  }
}

function getCursorOrCreate(connectionId: string): HTMLElement {
  let cursor: HTMLElement | null = document.getElementById(`cursor-${connectionId}`);

  if (cursor == null) {
    cursor = document.getElementById('cursor-template')!.cloneNode(true) as HTMLElement;
    cursor.id = `cursor-${connectionId}`;
    cursor.style.fill = COLORS[Math.floor(Math.random() * COLORS.length)];
    cursorsContainer.appendChild(cursor);
  }

  return cursor;
}

function deleteCursor(user: { clientId: string }) {
  const cursor = document.getElementById(`cursor-${user.clientId}`);
  if (cursor) {
    cursor.parentNode!.removeChild(cursor);
  }
}
