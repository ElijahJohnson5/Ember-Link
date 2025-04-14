import type { Metadata } from 'next';
import { Geist, Geist_Mono } from 'next/font/google';
import './globals.css';
import { SidebarInset, SidebarProvider } from '@/components/ui/sidebar';
import { AppSidebar } from '@/components/app-sidebar';
import { LayoutProvider } from './layout-provider';

const geistSans = Geist({
  variable: '--font-geist-sans',
  subsets: ['latin']
});

const geistMono = Geist_Mono({
  variable: '--font-geist-mono',
  subsets: ['latin']
});

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
    <html lang="en">
      <body className={`${geistSans.variable} ${geistMono.variable} antialiased`}>
        <LayoutProvider>
          <SidebarProvider>
            <AppSidebar />
            <SidebarInset>
              <main className="flex flex-col h-full">
                <div className="flex-grow h-full">{children}</div>
              </main>
            </SidebarInset>
          </SidebarProvider>
        </LayoutProvider>
      </body>
    </html>
  );
}
