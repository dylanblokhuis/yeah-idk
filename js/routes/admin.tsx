import { PostType, useRouteData } from '$lib';
import React from 'react'

export default function Admin() {
  const data = useRouteData<PostType[]>();

  return (
    <div>
      {data.map(type => (
        <div key={type.id}>
          <a href={`/admin/posts?type=${type.id}`}>
            {type.singular} - {type.plural}
          </a>
        </div>
      ))}
    </div>
  )
}
