[![MIT][s2]][l2] [![Latest Version][s1]][l1] [![docs][s3]][l3] [![Chat on Miaou][s4]][l4]

[s1]: https://img.shields.io/crates/v/flex-grow.svg
[l1]: https://crates.io/crates/flex-grow

[s2]: https://img.shields.io/badge/license-MIT-blue.svg
[l2]: LICENSE

[s3]: https://docs.rs/flex-grow/badge.svg
[l3]: https://docs.rs/flex-grow/

[s4]: https://miaou.dystroy.org/static/shields/room.svg
[l4]: https://miaou.dystroy.org/3


Tiny utility computing the allocation of a size among "children".

Typical use case: decide what columns to show in an UI, and what size to give to each column.

Each child can have a min and max size, be optional with a priority, have a `grow` factor.

Example:

```
use flex_grow::{Child, Container};

let container = Container::builder_in(50)
    .with_margin_between(1)
    .with(Child::new("name").clamp(5, 10))
    .with(Child::new("price").with_size(8).optional_with_priority(7))
    .with(Child::new("quantity").with_size(8).optional())
    .with(Child::new("total").with_size(8))
    .with(Child::new("comments").with_min(10).with_grow(2.0))
    .with(Child::new("vendor").with_size(60).optional_with_priority(9))
    .build()
    .unwrap();
assert_eq!(container.sizes(), vec![7, 8, 8, 8, 15, 0]);
```

You can give any argument to `Child::new`, it's stored in the child and returned by the `content()` method.


