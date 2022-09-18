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

export interface RouteErrors {
  errors: {
    level: string,
    message: string
  }[]
}