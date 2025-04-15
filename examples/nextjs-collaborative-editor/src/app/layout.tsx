import type { Metadata } from 'next';
import './globals.css';
import { SidebarInset, SidebarProvider } from '@/components/ui/sidebar';
import { AppSidebar } from '@/components/app-sidebar';
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
    <html lang="en">
      <body className="antialiased">
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
