import { createContext, useContext } from 'react';

export const RouteContext = createContext(globalThis.routeData);
export function useRouteData<T = any>(): T {
  return useContext(RouteContext);
}

export interface Post {
  id: string
  title: string
  content: string
  status: "draft" | "published"
  type: string
}


export interface PostType {
  id: string,
  singular: string,
  plural: string,
  path_prefix?: string,
}

export interface Posts {
  posts: Post[]
  post_type: PostType
}


export interface FlashError {
  level: "error" | "warning" | "info" | "success"
  message: string
}
