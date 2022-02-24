# Account Common

## Overview

Account Common defines a small set of types that need to be used in both Account
Manager and Account Handler.


## Key Dependencies

None


## Design

`error::AccountManagerError` defines an Error type implementing failure::Fail
and containing the most appropriate fuchsia.identity.account.Error to
communicate the error over FIDL.

`identifiers` defines more ergonomic Rust wrapper types for the FIDL identifiers
defined in fuchsia.identity.account.


## Future Work

No future work is currently planned in this crate. In the future, improvements
to the autogenerated FIDL bindings might allow us to use the autogenerated FIDL
types rather than the manually generated wrapper types in the identifier module.
