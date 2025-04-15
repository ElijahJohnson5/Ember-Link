import { Providers } from './providers';

export default async function DocumentLayout({
  children,
  params
}: Readonly<{
  children: React.ReactNode;
  params: Promise<{ document: string }>;
}>) {
  const { document } = await params;

  return <Providers document={document}>{children}</Providers>;
}
