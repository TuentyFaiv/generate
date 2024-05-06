pub static STYLES_CSS: &str = r#"@import "./NAME_PASCAL.responsive.css";

.NAME_LOWER {
}
"#;

pub static STYLES_CSS_RESPONSIVE: &str = r#"@media (min-width: 1024px) {
  .NAME_LOWER {
  }
}
@media (min-width: 1280px) {
  .NAME_LOWER {
  }
}
"#;

pub static STYLES_SASS: &str = r#"@import "@mixins";

.NAME_LOWER {
}

@import "./NAME_PASCAL.responsive.scss";
"#;

pub static STYLES_SASS_RESPONSIVE: &str = r#".NAME_LOWER {
  @include forsize(desktop-mid) {
  }
  @include forsize(desktop) {
  }
}
"#;

pub static STYLES_LESS: &str = r#".NAME_LOWER {
}

@import "./NAME_PASCAL.responsive.less";
"#;

pub static STYLES_LESS_RESPONSIVE: &str = r#".NAME_LOWER {
  @media (min-width: 1024px) {
  }
  @media (min-width: 1280px) {
  }
}
"#;

pub static STYLES_STYLUS: &str = r#".NAME_LOWER


@import "./NAME_PASCAL.responsive.styl"
"#;

pub static STYLES_STYLUS_RESPONSIVE: &str = r#"@media (min-width: 1024px)
  .NAME_LOWER
@media (min-width: 1280px)
  .NAME_LOWER
"#;

pub static STYLES_POSTCSS: &str = r#"@import "./NAME_PASCAL.responsive.postcss";

.NAME_LOWER {
}
"#;

pub static STYLES_POSTCSS_RESPONSIVE: &str = r#".NAME_LOWER {
  @media (min-width: 1024px) {
  }
  @media (min-width: 1280px) {
  }
}
"#;
