import { Post, useRouteData } from '$lib'
import React from 'react'

export default function Posts() {
  const data = useRouteData<Post[]>();

  return (
    <div>
      <h1>Posts - <a href="/admin/posts/create">Create post</a></h1>
      <div>
        {data.map(item => (
          <div key={item.id}>
            {JSON.stringify(item)}
          </div>
        ))}
      </div>
    </div>
  )
}
