pub static PAGE_TS: &str = r#"import { useState } from "react";
import { useTranslation } from "react-i18next";
import { useAppContext } from "@contexts";

import type { NAMEState } from "@typing/pages/NAME_LOWER";

import * as Styles from "./NAME.styles";

export default function NAMEPage() {
  const { t } = useTranslation("NAME_LOWER");
  const { global, dipatch } = useAppContext();
  const [state, setState] = useState<NAMEState>(null);

  return (
    <Styles.NAME>
    </Styles.NAME>
  );
}
"#;

pub static PAGE: &str = r#"import { useState } from "react";
import { useTranslation } from "react-i18next";
import { useAppContext } from "@contexts";

import * as Styles from "./NAME.styles";

export default function NAMEPage() {
  const { t } = useTranslation("NAME_LOWER");
  const { global, dipatch } = useAppContext();
  const [state, setState] = useState(null);

  return (
    <Styles.NAME>
    </Styles.NAME>
  );
}
"#;

pub static STYLES: &str = r#"import styled from "@emotion/styled";

import * as responsive from "./NAME.styles.responsive";

export const NAME = styled.div`
  ${responsive.NAME_LOWER}
`;
"#;

pub static STYLES_RESPONSIVE: &str = r#"import { css } from "@emotion/react";
import { forsize } from "@mixins";

export const NAME_LOWER = css`
${forsize({ size: "desktop-mid", content: css`

` })}
`;
"#;

pub static LOCALE: &str = r#"{
  "seo-title": "NAME"
}
"#;

pub static ROUTER: &str = r#"import { lazy, Suspense } from "react";
import { createBrowserRouter } from "react-router-dom";

import { Loader } from "@sharing/atoms";

// ROUTES

// ROUTER
export const router = createBrowserRouter([
  // NEXT_ROUTE
]);
"#;

pub static ROUTE: &str = r#"{
    path: "// ROUTE",
    element: (
      <Suspense fallback={<Loader type="page" portal />}>
        <NAME />
      </Suspense>
    )
  },
  // NEXT_ROUTE"#;

pub static PROPTYPES: &str = r#"// Change for interface if is an object
export type NAMEState = null;
"#;
