import { useRouteData } from '$lib'
import React from 'react'

export default function Home() {
  const data = useRouteData();

  console.log(JSON.stringify(data));

  return (
    <div>
      <h1>Home</h1>

      <a href="/admin">Go to admin</a>
    </div>
  )
}
