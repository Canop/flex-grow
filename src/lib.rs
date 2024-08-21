//!
//! Tiny utility computing the allocation of a size among "children".
//!
//! Typical use case: decide what columns to show in an UI, and what size to give to each column.
//!
//! Each child can have a min and max size, be optional with a priority, have a `grow` factor.
//!
//! Example:
//!
//! ```
//! use flex_grow::{Child, Container};
//!
//! let container = Container::builder_in(50)
//!     .with_margin_between(1)
//!     .with(Child::new("name").clamp(5, 10))
//!     .with(Child::new("price").with_size(8).optional_with_priority(7))
//!     .with(Child::new("quantity").with_size(8).optional())
//!     .with(Child::new("total").with_size(8))
//!     .with(Child::new("comments").with_min(10).with_grow(2.0))
//!     .with(Child::new("vendor").with_size(60).optional_with_priority(9))
//!     .build()
//!     .unwrap();
//! assert_eq!(container.sizes(), vec![7, 8, 8, 8, 15, 0]);
//! ```
//!
//! You can give any argument to `Child::new`, it's stored in the child and returned by the `content()` method.
//!

use std::fmt;

pub struct ContainerBuilder<C> {
    available: usize,
    margin_between: usize,
    children: Vec<Child<C>>,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum Optionality {
    #[default]
    Required,
    Optional {
        priority: usize, // bigger is more important
    },
}

pub struct Child<C> {
    content: C,
    constraints: ChildConstraints,
    size: Option<usize>, // None if not (yet) included
}

#[derive(Debug, Clone, Copy)]
pub struct ChildConstraints {
    pub min: usize,
    pub max: Option<usize>,
    pub optionality: Optionality,
    pub grow: f64,
}

impl Default for ChildConstraints {
    fn default() -> Self {
        ChildConstraints {
            min: 0,
            max: None,
            optionality: Optionality::default(),
            grow: 1.0,
        }
    }
}

pub struct Container<C> {
    pub children: Vec<Child<C>>,
}

#[derive(Debug, Clone)]
pub enum Error {
    NotEnoughSpace,
}
impl std::error::Error for Error {}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::NotEnoughSpace => write!(f, "Not enough space"),
        }
    }
}

impl ChildConstraints {}

impl<C> ContainerBuilder<C> {
    pub fn with_available(available: usize) -> Self {
        ContainerBuilder {
            available,
            children: Vec::new(),
            margin_between: 0,
        }
    }
    pub fn with_margin_between(mut self, margin: usize) -> Self {
        self.margin_between = margin;
        self
    }
    pub fn with(mut self, child: Child<C>) -> Self {
        self.add(child);
        self
    }
    pub fn add(&mut self, child: Child<C>) {
        self.children.push(child);
    }
    pub fn build(self) -> Result<Container<C>, Error> {
        let Self {
            mut available,
            mut children,
            margin_between,
        } = self;

        // first pass: we only add the required children. If their min size
        // is too big, we return an error.
        let mut added_children = 0;
        for child in &mut children {
            child.size = if child.is_optional() {
                None
            } else {
                let margin = if added_children > 0 {
                    margin_between
                } else {
                    0
                };
                if child.constraints.min + margin > available {
                    return Err(Error::NotEnoughSpace);
                }
                available -= child.constraints.min;
                available -= margin;
                added_children += 1;
                Some(child.constraints.min)
            };
        }

        // second pass: we add the optional children until we run out of space,
        // by priority
        let mut optional_children = children
            .iter_mut()
            .filter(|c| c.is_optional())
            .collect::<Vec<_>>();
        optional_children.sort_by_key(|c| {
            std::cmp::Reverse(match c.constraints.optionality {
                Optionality::Optional { priority } => priority,
                _ => 0,
            })
        });
        for child in optional_children {
            let margin = if added_children > 0 {
                margin_between
            } else {
                0
            };
            if child.constraints.min + margin > available {
                continue;
            }
            available -= child.constraints.min;
            available -= margin;
            added_children += 1;
            child.size = Some(child.constraints.min);
        }

        // then we distribute the remaining space to the growable children
        let mut growths = vec![0.0; children.len()];
        let mut sum_growths = 0.0;
        for (i, child) in children.iter().enumerate() {
            let Some(size) = child.size else {
                continue;
            };
            growths[i] = child.constraints.grow
                * (match child.constraints.max {
                    None => available,
                    Some(max) => max - size,
                } as f64);
            sum_growths += growths[i];
        }
        for i in 0..children.len() {
            let Some(size) = children[i].size else {
                continue;
            };
            let growth = growths[i] as f64 * (available as f64 / sum_growths);
            available -= growth as usize;
            children[i].size = Some(size + growth as usize);
        }

        // Due to down rounding, it's probable that there's some available space left.
        while available > 0 {
            let mut given = 0;
            for child in &mut children {
                let Some(size) = child.size else {
                    continue;
                };
                if child.constraints.max.map_or(true, |max| size < max) {
                    child.size = Some(size + 1);
                    given += 1;
                    available -= 1;
                    if available == 0 {
                        break;
                    }
                }
            }
            if given == 0 {
                break;
            }
        }

        let con = Container { children };
        Ok(con)
    }
}

impl<C> Child<C> {
    pub fn new(content: C) -> Self {
        let constraints = ChildConstraints::default();
        Child {
            content,
            constraints,
            size: None,
        }
    }
    pub fn content(&self) -> &C {
        &self.content
    }
    pub fn optional(self) -> Self {
        self.optional_with_priority(0)
    }
    pub fn optional_with_priority(mut self, priority: usize) -> Self {
        self.constraints.optionality = Optionality::Optional { priority };
        self
    }
    pub fn with_min(mut self, min: usize) -> Self {
        self.constraints.min = min;
        self
    }
    pub fn with_max(mut self, max: usize) -> Self {
        self.constraints.max = Some(max);
        self
    }
    pub fn clamp(mut self, min: usize, max: usize) -> Self {
        self.constraints.min = min;
        self.constraints.max = Some(max);
        self
    }
    pub fn with_size(mut self, size: usize) -> Self {
        self.constraints.min = size;
        self.constraints.max = Some(size);
        self
    }
    pub fn with_grow(mut self, grow: f64) -> Self {
        self.constraints.grow = grow;
        self
    }
    pub fn constraints(&self) -> ChildConstraints {
        self.constraints
    }
    fn is_optional(&self) -> bool {
        matches!(self.constraints.optionality, Optionality::Optional { .. })
    }
    /// Return the size, if the child is included in the container, or none
    /// if there wasn't enough space to include it
    pub fn size(&self) -> Option<usize> {
        self.size
    }
}

impl<C> Container<C> {
    pub fn builder_in(available: usize) -> ContainerBuilder<C> {
        ContainerBuilder::with_available(available)
    }
    /// Return the sizes of the children, in the order they were added,
    /// with 0 for non-included children
    pub fn sizes(&self) -> Vec<usize> {
        self.children
            .iter()
            .map(|sc| sc.size.unwrap_or(0))
            .collect()
    }
    pub fn children(&self) -> &[Child<C>] {
        &self.children
    }
    pub fn to_children(self) -> Vec<Child<C>> {
        self.children
    }
}
