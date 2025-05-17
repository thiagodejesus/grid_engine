use std::{thread, time};

use grid_engine::grid_engine::GridEngine;

enum Interaction {
    AddItem(String, usize, usize, usize, usize),
    MoveItem(String, usize, usize),
    RemoveItem(String),
    InvalidInteraction(String),
}

impl Interaction {
    fn from_str(input: &str) -> Interaction {
        // Should match input starts with
        let mut parts = input.split_whitespace();
        let action = parts.next().unwrap_or("");

        match action {
            "add" => {
                println!("{}", input);
                let id = parts.next().expect("Expect id").to_string();

                let x = parts
                    .next()
                    .expect("Expect X")
                    .parse()
                    .expect("Expect x to be number");
                let y = parts
                    .next()
                    .expect("Expect Y")
                    .parse()
                    .expect("Expect y to be number");
                let w = parts
                    .next()
                    .expect("Expect W")
                    .parse()
                    .expect("Expect w to be number");
                let h = parts
                    .next()
                    .expect("Expect H")
                    .parse()
                    .expect("Expect h to be number");

                Interaction::AddItem(id, x, y, w, h)
            }
            "rm" => {
                let id = parts.next().expect("Expect ID");
                Interaction::RemoveItem(id.to_string())
            }
            "mv" => {
                let id = parts.next().expect("Expect ID");
                let x = parts
                    .next()
                    .expect("Expect X")
                    .parse()
                    .expect("Expect x to be number");
                let y = parts
                    .next()
                    .expect("Expect Y")
                    .parse()
                    .expect("Expect y to be number");
                Interaction::MoveItem(id.to_string(), x, y)
            }
            _ => Interaction::InvalidInteraction(input.to_string()),
        }
    }
}

#[derive(Clone, Debug)]
struct GridContent {
    name: String,
}

impl Default for GridContent {
    fn default() -> Self {
        GridContent {
            name: "0".to_string(),
        }
    }
}

impl std::fmt::Display for GridContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

fn get_grid_formatted(grid_engine: &GridEngine, cell_space: u8) -> String {
    let mut grid_str = String::new();
    grid_str.push_str("  ");
    for i in 0..grid_engine.get_inner_grid().cols() {
        grid_str.push_str(&format!(" {} ", i));
    }
    grid_str.push_str("\n");

    grid_engine
        .get_inner_grid()
        .iter_rows()
        .enumerate()
        .for_each(|(row_number, row)| {
            row.enumerate().for_each(|(index, cell)| {
                if index == 0 {
                    grid_str.push_str(&format!("{:0>2}", row_number));
                }
                return match cell {
                    Some(item) => {
                        grid_str.push_str(&format!("[{}]", item));
                    }
                    None => {
                        grid_str.push_str(&format!("[{}]", " ".repeat(cell_space as usize)));
                    }
                };
            });
            grid_str.push_str("\n");
        });

    grid_str
}

fn print_grid(grid: &GridEngine) {
    // print!("\x1B[2J\x1B[1;1H");
    println!("Printing the grid");
    println!("{}", get_grid_formatted(grid, 1));
}

fn handle_interaction(grid: &mut GridEngine, interaction: Interaction) {
    match interaction {
        Interaction::AddItem(id, x, y, w, h) => {
            println!("Adding item to the grid");
            grid.add_item(id, x, y, w, h).unwrap();
        }
        Interaction::RemoveItem(id) => {
            println!("Removing item {} from the grid", &id);
            grid.remove_item(&id).unwrap();
        }
        Interaction::MoveItem(id, x, y) => {
            println!("Moving item {} to ({}, {})", &id, x, y);

            if x == 4 && y == 6 {
                println!("Moving item {} to ({}, {})", &id, x, y);
            }

            grid.move_item(&id, x, y).unwrap();
        }
        Interaction::InvalidInteraction(instruction) => {
            println!("Invalid interaction: {}", instruction);
        }
    }
    print_grid(grid);
}

// fn interactive_mode() {
//     println!("Grid App");

//     let mut grid = GridEngine::new(8, 12);

//     loop {
//         // Reads some input from the user and prints it back
//         let mut input = String::new();
//         std::io::stdin().read_line(&mut input).unwrap();

//         let input = input.trim();

//         handle_interaction(&mut grid, Interaction::from_str(input));
//     }
// }

fn scripted_mode() {
    println!("Grid App");

    let mut grid = GridEngine::new(10, 12);

    grid.events.add_changes_listener(Box::new(|event| {
        println!("Event triggered: {:?}", event);
        // grid.
        // grid.print_grid();
    }));

    let instructions = vec![
        "add a 2 2 2 4 1",
        "add b 4 2 2 4 2",
        "add c 0 2 2 2",
        "rm b",
        "add d 4 2 2 3 0",
        "add e 2 2 2 4 1",
        "add f 2 2 2 4 1",
        "rm f",
        "add g 2 2 2 4 1",
        "rm a",
        "mv c 1 0",
        "mv c 2 0",
        "mv c 2 2",
        "mv c 3 2",
        "mv c 4 10",
        "mv c 4 6",
        "mv d 1 1",
        "mv c 4 6",
        "mv e 2 15",
        "mv c 2 17",
        "mv c 4 6",
        "mv e 1 2",
    ];

    for instruction in instructions {
        handle_interaction(&mut grid, Interaction::from_str(instruction));
        thread::sleep(time::Duration::from_millis(250))
    }
}

fn main() {
    // interactive_mode();
    scripted_mode();
}
