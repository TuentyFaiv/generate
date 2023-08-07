pub static HOC_TS: &str = r#"import { forwardRef } from "react";
import { useDatas } from "@hooks";

import type { ComponentType } from "react";
import type { HOCNAME, HOCNAMEProps } from "@typing/hocs/withNAME";

export function withNAME<T extends HOCNAME = HOCNAME>(Component: ComponentType<T>) {
  const WithNAME = forwardRef((props: Omit<T, keyof HOCNAME>, ref) => {
    const { data = {}, ...newProps } = props as T & HOCNAMEProps;
    const datas = useDatas(data);

    return (
      <Component
        {...(newProps as unknown as T)}
        ref={ref}
        datas={datas}
      />
    );
  });

  WithNAME.displayName = `withNAME(${Component.displayName ?? Component.name})`;

  return WithNAME;
}
"#;
pub static HOC: &str = r#"import { forwardRef } from "react";
import { useDatas } from "@hooks";

export function withNAME(Component) {
  const WithNAME = forwardRef((props, ref) => {
    const { data = {}, ...newProps } = props;
    const datas = useDatas(data);

    return (
      <Component
        {...newProps}
        ref={ref}
        datas={datas}
      />
    );
  });

  WithNAME.displayName = `withNAME(${Component.displayName ?? Component.name})`;

  return WithNAME;
}
"#;

pub static PROPTYPES: &str = r#"import type { ObjStrCommon } from "@typing/globals/types";

export interface HOCNAME {
  datas: ObjStrCommon;
}

export interface HOCNAMEProps {
  data?: ObjStrCommon;
}
"#;