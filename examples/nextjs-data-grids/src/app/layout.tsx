import type { Metadata } from 'next';
import './globals.css';
import { LayoutProvider } from './layout-provider';

export const metadata: Metadata = {
  title: 'Notion Clone',
  description: 'Clone of Notion using NoteBlock and EmberLink'
};

export default function RootLayout({
  children
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" className="h-full">
      <body className="antialiased h-full">
        <LayoutProvider>
          <main className="flex flex-col h-full">
            <div className="flex-grow h-full">{children}</div>
          </main>
        </LayoutProvider>

        <div id="portal" />
      </body>
    </html>
  );
}
