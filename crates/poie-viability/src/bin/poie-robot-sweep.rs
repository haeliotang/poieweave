// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S3 robot-count sweep for three concurrent repairs with a common deadline.

use std::process::ExitCode;

fn main() -> ExitCode {
    let repairs = [3.0_f64, 2.0, 2.0];
    let deadline_h = 4.0;
    println!("repairs\tdeadline_h\trobots\tmakespan_h\tmeets");
    for robots in 1..=3usize {
        let mut free = vec![0.0_f64; robots];
        for repair in repairs {
            let (index, _) = free.iter().enumerate().min_by(|left, right| left.1.total_cmp(right.1)).expect("robots");
            free[index] += repair;
        }
        let makespan = free.iter().copied().fold(0.0_f64, f64::max);
        println!("{}\t{deadline_h:.0}\t{robots}\t{makespan:.0}\t{}", repairs.len(), makespan <= deadline_h);
    }
    ExitCode::SUCCESS
}
