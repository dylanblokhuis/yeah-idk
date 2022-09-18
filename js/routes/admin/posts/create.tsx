import { RouteErrors, useRouteData } from '$lib'
import React from 'react'

export default function Create() {
  const data = useRouteData<RouteErrors>();

  return (
    <div>
      <h1>Create post</h1>

      <form action="/admin/posts" method="post" style={{ display: 'flex', flexDirection: 'column', alignItems: 'start' }}>
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
