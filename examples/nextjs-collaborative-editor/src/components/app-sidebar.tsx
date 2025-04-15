'use client';

import * as React from 'react';

import {
  Sidebar,
  SidebarContent,
  SidebarGroup,
  SidebarGroupAction,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarRail
} from '@/components/ui/sidebar';
import { Plus } from 'lucide-react';
import { useCreateDocumentAction } from '@/hooks/use-create-document-action';
import Link from 'next/link';
import { useDocuments } from '@/hooks/use-documents';
import { usePathname } from 'next/navigation';

const data = {
  navMain: [
    {
      title: 'Documents',
      items: []
    }
  ]
};

export function AppSidebar({ ...props }: React.ComponentProps<typeof Sidebar>) {
  const documents = useDocuments();
  const currentPath = usePathname();

  const createDocumentAction = useCreateDocumentAction();

  return (
    <Sidebar {...props}>
      <SidebarHeader>
        <Link href="/">Notion Clone</Link>
      </SidebarHeader>
      <SidebarContent>
        {/* We create a SidebarGroup for each parent. */}
        {data.navMain.map((item) => (
          <SidebarGroup key={item.title}>
            <SidebarGroupLabel>{item.title}</SidebarGroupLabel>
            <SidebarGroupAction onClick={createDocumentAction} title="Create Document">
              <Plus /> <span className="sr-only">Create Doucment</span>
            </SidebarGroupAction>
            <SidebarGroupContent>
              <SidebarMenu>
                {documents.current.map((item) => (
                  <SidebarMenuItem key={item.id}>
                    <SidebarMenuButton
                      asChild
                      isActive={
                        currentPath === (item.title ? `/${item.title}-${item.id}` : `/${item.id}`)
                      }
                    >
                      <Link href={item.title ? `/${item.title}-${item.id}` : `/${item.id}`}>
                        {item.title ?? 'New Page'}
                      </Link>
                    </SidebarMenuButton>
                  </SidebarMenuItem>
                ))}
              </SidebarMenu>
            </SidebarGroupContent>
          </SidebarGroup>
        ))}
      </SidebarContent>
      <SidebarRail />
    </Sidebar>
  );
}
