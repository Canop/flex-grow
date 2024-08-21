use flex_grow::*;

fn main() {
    let terminal_width = crossterm::terminal::size()
        .unwrap().0 as usize;

    let container = Container::builder_in(terminal_width)
        .with_margin_between(1)
        .with(Child::new("name").clamp(5, 10))
        .with(Child::new("price").with_size(8).optional_with_priority(7))
        .with(Child::new("quantity").with_size(8).optional_with_priority(8))
        .with(Child::new("total").with_size(8))
        .with(Child::new("comments").with_min(10))
        .with(Child::new("vendor").with_size(60).optional_with_priority(9))
        .build()
        .unwrap();
    let sum_sizes = container.sizes().iter().sum::<usize>();
    assert!(sum_sizes <= terminal_width);

    println!("terminal_width: {}", terminal_width);
    // name row
    let mut added = false;
    for child in &container.children {
        if let Some(size) = child.size() {
            if added {
                print!("|");
            } else {
                added = true;
            }
            let name = child.content();
            print!("{name:^size$}");
        }
    }
    println!();
    // size row
    let mut added = false;
    for child in &container.children {
        if let Some(size) = child.size() {
            if added {
                print!("|");
            } else {
                added = true;
            }
            print!("{size:^size$}");
        }
    }
    println!();
}
