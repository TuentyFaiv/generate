pub static LAYOUT_TS_STYLED: &str = r#"import { useState } from "react";
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

pub static LAYOUT_STYLED: &str = r#"import { useState } from "react";
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

pub static LAYOUT_TS_CSS: &str = r#"import { useState } from "react";
import { Outlet } from "react-router-dom";

import type { NAME_PASCALLayoutState } from "@typing/layouts/NAMESPACE";

import "@NAMESPACE/styles";

export default function NAME_PASCALLayout() {
  const [state, setState] = useState<NAME_PASCALLayoutState>(null);

  return (
    <div className="NAME_LOWER">
      <Outlet />
    </div>
  );
}
"#;

pub static LAYOUT_CSS: &str = r#"import { useState } from "react";
import { Outlet } from "react-router-dom";

import "@NAMESPACE/styles";

export default function NAME_PASCALLayout() {
  const [state, setState] = useState(null);

  return (
    <div className="NAME_LOWER">
      <Outlet />
    </div>
  );
}
"#;

pub static STYLES_EMOTION: &str = r#"import styled from "@emotion/styled";

import * as responsive from "./NAME_PASCALLayout.styles.responsive";

export const NAME_PASCAL = styled.div`
  ${responsive.NAMESPACE}
`;
"#;

pub static STYLES_EMOTION_RESPONSIVE: &str = r#"import { css } from "@emotion/react";
import { forsize } from "@mixins";

export const NAME_LOWER = forsize({
  "desktop-mid": css``,
  desktop: css``,
});
"#;

pub static STYLES_STYLED: &str = r#"import styled from "styled-components";

import * as responsive from "./NAME_PASCALLayout.styles.responsive";

export const NAME_PASCAL = styled.div`
  ${responsive.NAMESPACE}
`;
"#;

pub static STYLES_STYLED_RESPONSIVE: &str = r#"import { css } from "styled-components";
import { forsize } from "@mixins";

export const NAME_LOWER = forsize({
  "desktop-mid": css``,
  desktop: css``,
});
"#;

pub static PROPTYPES: &str = r#"// Change for interface if is an object
export type NAME_PASCALLayoutState = null;
"#;

pub static BARREL_STYLES_STYLED: &str = r#"export * as Layout from "./NAME_PASCALLayout.styles";
"#;

pub static BARREL_STYLES_CSS: &str = r#"import "./NAME_PASCALLayoutEXT_STYLES";
"#;