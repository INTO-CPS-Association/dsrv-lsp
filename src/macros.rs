/*
 * Copyright (c) 2026 Emilie Bang Holmberg (github.com/EmmiPigen).
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License.
 *
 * This project utilizes the 'trustworthiness-checker' crate, which is
 * property of the INTO-CPS Association and used under the ICAPL (GPL Mode).
 */

#[macro_export]
macro_rules! async_test {
    (
      $(#[$attr:meta])*
      async fn $name:ident $($rest:tt)*
    ) => {
      $(#[$attr])*
      #[test_log::test(macro_rules_attribute::apply(smol_macros::test))]
      async fn $name $($rest)*
    };
}