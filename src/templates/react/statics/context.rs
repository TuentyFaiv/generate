pub static CONTEXT_TS: &str = r#"import { createContext, useContext, useReducer, useMemo } from "react";

import type { ContextNAME, ContextNAMEProvider, ContextNAMEState } from "@typing/contexts/NAME_LOWER";

import reducer from "./reducer";

const NAMEContext = createContext<ContextNAME>({
  state: null,
  dispatch: () => {}
});

export function NAMEProvider({ children }: ContextNAMEProvider) {
  const storageItem = localStorage.getItem("item");

  const initialState: ContextNAMEState = null;

  const [state, dispatch] = useReducer(reducer, initialState);

  const contextValue = useMemo(() => ({ state, dispatch }), [global]);

  return (
    <NAMEContext.Provider value={contextValue}>
      {children}
    </NAMEContext.Provider>
  );
}

export const useNAMEContext = () => useContext(NAMEContext);
"#;

pub static CONTEXT: &str = r#"import { createContext, useContext, useReducer, useMemo } from "react";

import reducer from "./reducer";
export { Actions } from "./reducer";

const NAMEContext = createContext({
  state: null,
  dispatch: () => {}
});

export function NAMEProvider({ children }) {
  const storageItem = localStorage.getItem("item");

  const initialState = null;

  const [state, dispatch] = useReducer(reducer, initialState);

  const contextValue = useMemo(() => ({ state, dispatch }), [global]);

  return (
    <NAMEContext.Provider value={contextValue}>
      {children}
    </NAMEContext.Provider>
  );
}

export const useNAMEContext = () => useContext(NAMEContext);
"#;

pub static REDUCER_TS: &str = r#"import { Actions } from "@typing/contexts/NAME_LOWER";

import type { ContextNAMEReducerAction, ContextNAMEState } from "@typing/contexts/NAME_LOWER";

export default function reducer(state: ContextNAMEState, action: ContextNAMEReducerAction): ContextNAMEState {
  switch (action.type) {
    default:
      return state;
  }
}
"#;

pub static REDUCER: &str = r#"
export const Actions = {
  CHANGE: "CHANGE"
};

export default function reducer(state, action) {
  switch (action.type) {
    default:
      return state;
  }
}
"#;

pub static PROPTYPES: &str = r#"import type { Dispatch, ReactNode } from "react";

export interface ContextNAME {
  state: ContextNAMEState;
  dispatch: Dispatch<ContextNAMEReducerAction>;
};

// Change for interface if is an object
export type ContextNAMEState = null;

export interface ContextNAMEProvider {
  children: ReactNode;
};

export type ContextNAMEReducerAction = ChangeAction;

interface ChangeAction {
  type: NAMEActions;
  payload: ContextNAMEState;
}

export enum NAMEActions {
  CHANGE = "CHANGE"
}
"#;