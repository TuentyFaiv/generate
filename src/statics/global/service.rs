pub static SERVICE_TS: &str = r#"import { api } from "@services/globals";

import type {
  NAME_PASCALBody,
  NAME_PASCALParams,
  NAME_PASCALConfig,
  NAME_PASCALReturn,
} from "@typing/services/NAMESPACE";

export async function NAME_CAMEL({ body, params, signal }: NAME_PASCALConfig): Promise<NAME_PASCALReturn> {
  const { payload } = await api.post<NAME_PASCALBody, NAME_PASCALReturn, NAME_PASCALParams>("/", body, {
    params,
    signal,
  });

  return payload;
}
"#;
pub static SERVICE: &str = r#"import { api } from "@services/globals";

export async function NAME_CAMEL({ body, params, signal }) {
  const { payload } = await api.post("/", body, {
    params,
    signal,
  });

  return payload;
}
"#;

pub static PROPTYPES_IMPORTS: &str = r#"import type { ServiceGeneralConfig } from "@typing/services/globals";

/* PROPTYPES */
"#;

pub static PROPTYPES: &str = r#"// NAME_PASCAL Service
// Change for interface if is an object
export type NAME_PASCALBody = null;

// Change for interface if is an object
export type NAME_PASCALParams = null;

// Change for interface if is an object
export type NAME_PASCALReturn = null;

export type NAME_PASCALConfig = ServiceGeneralConfig<NAME_PASCALBody, NAME_PASCALParams>;

/* NEXT_TYPE */"#;

pub static INSTANCES: &str = r#"import { Http } from "@tuentyfaiv/http";
import config from "@config";

export const api = Http.create(config.api, {
  storage: localStorage
});
export const registry = Http.create(config.registry_url, {
  params: {
    key: config.registry_key,
  },
});

export function setAuth(token) {
  api.setAuth(token);
}

export function setLang(lang) {
  api.setLang(lang);
}
"#;

pub static INSTANCES_TS: &str = r#"import { Http } from "@tuentyfaiv/http";
import config from "@config";

export const api = Http.create(config.api, {
  storage: localStorage
});
export const registry = Http.create(config.registry_url, {
  params: {
    key: config.registry_key,
  },
});

export function setAuth(token: string) {
  api.setAuth(token);
}

export function setLang(lang: string) {
  api.setLang(lang);
}
"#;
