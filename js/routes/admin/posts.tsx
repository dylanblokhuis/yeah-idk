import { useRouteData } from '$lib'
import React from 'react'

export default function Posts() {
  const data = useRouteData();

  return (
    <div><h1>Posts</h1>
      <div>
        {data.map(item => (
          <div key={item.id}>
            {item.name}
          </div>
        ))}
      </div>
    </div>
  )
}
