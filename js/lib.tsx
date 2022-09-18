import { createContext, useContext } from 'react';

export const RouteContext = createContext(globalThis.routeData);
export function useRouteData() {
  return useContext(RouteContext);
}