import './globals.css';
import type { Metadata } from 'next';
export const metadata: Metadata = { title: 'sku media Split Merge', description: 'Cắt và ghép video' };
export default function RootLayout({children}: Readonly<{children: React.ReactNode}>) { return <html lang="vi"><body>{children}</body></html>; }
