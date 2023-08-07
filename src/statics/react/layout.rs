pub static LAYOUT_TS: &str = r#"import { useState } from "react";
import { Outlet } from "react-router-dom";

import type { NAME_PASCALLayoutState } from "@typing/layouts/NAMESPACE";

import { Layout } from "@NAMESPACE/styles";

export default function NAME_PASCALLayout() {
  const [state, setState] = useState<NAME_PASCALLayoutState>(null);

  return (
    <Layout.NAME_PASCAL>
      <Outlet />
    </Layout.NAME_PASCAL>
  );
}
"#;

pub static LAYOUT: &str = r#"import { useState } from "react";
import { Outlet } from "react-router-dom";

import { Layout } from "@NAMESPACE/styles";

export default function NAME_PASCALLayout() {
  const [state, setState] = useState(null);

  return (
    <Layout.NAME_PASCAL>
      <Outlet />
    </Layout.NAME_PASCAL>
  );
}
"#;

pub static STYLES: &str = r#"import styled from "@emotion/styled";

import * as responsive from "./NAME_PASCALLayout.styles.responsive";

export const NAME_PASCAL = styled.div`
  ${responsive.NAMESPACE}
`;
"#;

pub static STYLES_RESPONSIVE: &str = r#"import { css } from "@emotion/react";
import { forsize } from "@mixins";

export const NAME_LOWER = forsize({
  "desktop-mid": css``,
  desktop: css``,
});
"#;

pub static PROPTYPES: &str = r#"// Change for interface if is an object
export type NAME_PASCALLayoutState = null;
"#;
