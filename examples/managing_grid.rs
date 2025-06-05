use grid_engine::grid_engine::GridEngine;

#[derive(Clone, Debug)]
struct GridContent {
    id: String,
}

impl Default for GridContent {
    fn default() -> Self {
        GridContent {
            id: "0".to_string(),
        }
    }
}

impl std::fmt::Display for GridContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

fn print_grid(grid: &GridEngine) {
    let mut grid_str_formatted = String::new();
    grid_str_formatted.push_str("  ");
    for i in 0..grid.get_inner_grid().cols() {
        grid_str_formatted.push_str(&format!(" {} ", i));
    }
    grid_str_formatted.push('\n');

    grid.get_inner_grid()
        .iter_rows()
        .enumerate()
        .for_each(|(row_number, row)| {
            row.enumerate().for_each(|(index, cell)| {
                if index == 0 {
                    grid_str_formatted.push_str(&format!("{:0>2}", row_number));
                }
                match cell {
                    Some(item) => {
                        grid_str_formatted.push_str(&format!("[{}]", item));
                    }
                    None => {
                        grid_str_formatted.push_str(&format!("[{}]", " "));
                    }
                };
            });
            grid_str_formatted.push('\n');
        });

    println!("{}", grid_str_formatted);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Grid App");

    let mut grid = GridEngine::new(10, 12);

    grid.events_mut().add_changes_listener(|event| {
        println!("Event triggered: {:#?}", event);
    })?;

    grid.add_item("a".to_string(), 2, 2, 2, 4).unwrap();
    print_grid(&grid);
    grid.add_item("b".to_string(), 4, 2, 2, 4).unwrap();
    print_grid(&grid);
    grid.add_item("c".to_string(), 0, 2, 2, 2).unwrap();
    print_grid(&grid);
    grid.remove_item("b").unwrap();
    print_grid(&grid);
    grid.move_item("a", 1, 0).unwrap();
    print_grid(&grid);

    Ok(())
}
