import { FlashError, PostType, useRouteData } from '$lib'
import React from 'react'

export interface CreateRoute {
  errors: FlashError[],
  post_type: PostType
}

export default function Create() {
  const data = useRouteData<CreateRoute>();

  return (
    <div>
      <h1>Create {data.post_type.singular}</h1>

      <form action="/admin/posts" method="post" style={{ display: 'flex', flexDirection: 'column', alignItems: 'start' }}>
        <input type="hidden" name="post_type" value={data.post_type.id} />
        <input type="text" name="title" placeholder="Title" />
        <textarea name="content" placeholder="Content"></textarea>

        {data.errors.map(error => (
          <ul>
            <li><b>{error.level}</b>: {error.message}</li>
          </ul>
        ))}

        <button type="submit">Add</button>
      </form>
    </div>
  )
}
