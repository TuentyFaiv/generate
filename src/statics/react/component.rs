pub static PROPTYPES: &str = r#"import type { ReactNode } from "react";

export interface Props {
  children: ReactNode;
}
"#;

pub static STYLES_EMOTION: &str = r#"import styled from "@emotion/styled";

import * as responsive from "./NAME_PASCAL.styles.responsive";

export const NAME_PASCAL = styled.div`
  ${responsive.NAME_LOWER}
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

import * as responsive from "./NAME_PASCAL.styles.responsive";

export const NAME_PASCAL = styled.div`
  ${responsive.NAME_LOWER}
`;
"#;

pub static STYLES_STYLED_RESPONSIVE: &str = r#"import { css } from "styled-components";
import { forsize } from "@mixins";

export const NAME_LOWER = forsize({
  "desktop-mid": css``,
  desktop: css``,
});
"#;

pub static COMPONENT_CSS: &str = r#"import "./NAME_PASCALEXT_STYLES";

export default function NAME_PASCAL({ children }) {
  return (
    <div className="NAME_LOWER">
      {children}
    </div>
  );
}
"#;

pub static COMPONENT_TS_CSS: &str = r#"import type { Props } from "./NAME_PASCAL.proptypes";

import "./NAME_PASCALEXT_STYLES";

export default function NAME_PASCAL({ children }: Props) {
  return (
    <div className="NAME_LOWER">
      {children}
    </div>
  );
}
"#;

pub static COMPONENT_STYLED: &str = r#"import * as Styles from "./NAME_PASCAL.styles";

export default function NAME_PASCAL({ children }) {
  return (
    <Styles.NAME_PASCAL>
      {children}
    </Styles.NAME_PASCAL>
  );
}
"#;

pub static COMPONENT_TS_STYLED: &str = r#"import type { Props } from "./NAME_PASCAL.proptypes";

import * as Styles from "./NAME_PASCAL.styles";

export default function NAME_PASCAL({ children }: Props) {
  return (
    <Styles.NAME_PASCAL>
      {children}
    </Styles.NAME_PASCAL>
  );
}
"#;