use colored::Colorize as _;
use libium::{add::Error, iter_ext::IterExt as _};
use std::collections::HashMap;

pub fn display_successes_failures(successes: &[String], failures: Vec<(String, Error)>) -> bool {
    if !successes.is_empty() {
        println!(
            "{} {}",
            "Successfully added".green(),
            successes.iter().map(|s| s.bold()).display(", ")
        );

    // No need to print the ID again if there is only one
    } else if failures.len() == 1 {
        let err = &failures[0].1;
        return if matches!(err, libium::add::Error::AlreadyAdded) {
            println!("{}", err.to_string().yellow());
            false
        } else {
            println!("{}", err.to_string().red());
            true
        };
    }

    let mut grouped_errors = HashMap::new();

    for (id, error) in failures {
        grouped_errors
            .entry(error.to_string())
            .or_insert_with(Vec::new)
            .push(id);
    }

    let pad_len = grouped_errors
        .keys()
        .map(String::len)
        .max()
        .unwrap_or(0)
        .clamp(0, 50);

    let mut exit_error = false;
    for (err, ids) in grouped_errors {
        println!(
            "{:pad_len$}: {}",
            // Change already added into a warning
            if err == libium::add::Error::AlreadyAdded.to_string() {
                err.yellow()
            } else {
                exit_error = true;
                err.red()
            },
            ids.iter().map(|s| s.italic()).display(", ")
        );
    }

    exit_error
}
