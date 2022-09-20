import { useRouteData } from '$lib'
import React from 'react'

export default function Post() {
  const data = useRouteData();
  return (
    <div>Post: {JSON.stringify(data)}</div>
  )
}
