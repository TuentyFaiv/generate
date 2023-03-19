pub static LAYOUT_TS: &str = r#"import { useState } from "react";
import { Outlet } from "react-router-dom";

import type { NAMELayoutState } from "@typing/layouts/NAME_LOWER";

import { Layout } from "@NAME_LOWER/styles";

export default function NAMELayout() {
  const [state, setState] = useState<NAMELayoutState>(null);

  return (
    <Layout.NAME>
      <Outlet />
    </Layout.NAME>
  );
}
"#;

pub static LAYOUT: &str = r#"import { useState } from "react";
import { Outlet } from "react-router-dom";

import { Layout } from "@NAME_LOWER/styles";

export default function NAMELayout() {
  const [state, setState] = useState(null);

  return (
    <Layout.NAME>
      <Outlet />
    </Layout.NAME>
  );
}
"#;

pub static STYLES_IMPORT: &str = r#"export * as Layout from "./NAMELayout.styles";
"#;

pub static STYLES: &str = r#"import styled from "@emotion/styled";

import * as responsive from "./NAMELayout.styles.responsive";

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

pub static PROPTYPES: &str = r#"// Change for interface if is an object
export type NAMELayoutState = null;
"#;
