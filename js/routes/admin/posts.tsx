import { Post, Posts, useRouteData } from '$lib'
import React from 'react'

export default function Posts() {
  const data = useRouteData<Posts>();

  return (
    <div>
      <a href="/admin">Back to overview</a>
      <h1>{data.post_type.plural} - <a href={`/admin/posts/create?type=${data.post_type.id}`}>Create {data.post_type.singular}</a></h1>
      <div>
        {data.posts.map(item => (
          <div key={item.id}>
            {JSON.stringify(item)}
          </div>
        ))}
      </div>
    </div>
  )
}
