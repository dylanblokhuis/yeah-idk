import React from 'react';

export default function Root({ children }: { children: React.ReactNode }) {
  return (
    <html>
      <head>
        <title>Hello World</title>
      </head>
      <body>
        {children}
      </body>
    </html>
  )
}
