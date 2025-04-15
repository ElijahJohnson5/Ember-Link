'use client';

import dynamic from 'next/dynamic';

export const BlockNote = dynamic(async () => (await import('./block-note')).BlockNote, { ssr: false });
