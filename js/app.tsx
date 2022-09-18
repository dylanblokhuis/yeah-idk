import React from 'react';
import { renderToStaticMarkup } from "react-dom"
import Root from "./root";
// @ts-expect-error
import Route from "$route";
import { RouteContext } from '$lib';

renderToStaticMarkup(
  <RouteContext.Provider value={globalThis.routeData}>
    <Root children={<Route />} />
  </RouteContext.Provider>
)