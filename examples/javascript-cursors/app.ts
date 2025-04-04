import { createClient, User } from '@ember-link/core';

declare global {
  interface EmberLink {
    Presence: {
      cursor: {
        x: number;
        y: number;
      } | null;
    };
  }
}

const client = createClient({
  baseUrl: 'https://ember-link-sandbox.onrender.com'
});

const { channel } = client.joinChannel('test');

const cursorsContainer = document.getElementById('cursors-container')!;
const text = document.getElementById('text')!;

channel.events.subscribe('presence', (presence) => {
  const cursor: { x: number; y: number } | null =
    (presence?.cursor as { x: number; y: number }) ?? null;

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
    cursor: { x: Math.round(event.clientX), y: Math.round(event.clientY) }
  });
});

document.addEventListener('pointerleave', () => {
  channel.updatePresence({ cursor: null });
});

const COLORS = ['#DC2626', '#D97706', '#059669', '#7C3AED', '#DB2777'];

// Update cursor position based on user presence
function updateCursor(user: User) {
  const cursor = getCursorOrCreate(user.clientId);

  if (user.cursor) {
    cursor.style.transform = `translateX(${user.cursor.x}px) translateY(${user.cursor.y}px)`;
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

function deleteCursor(user: User) {
  const cursor = document.getElementById(`cursor-${user.clientId}`);
  if (cursor) {
    cursor.parentNode!.removeChild(cursor);
  }
}
