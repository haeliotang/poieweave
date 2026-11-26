// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later
use poie_viability::engine::{FileAuthorizationLedger, LedgerClaim};
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        std::process::exit(2);
    }
    let ledger = FileAuthorizationLedger::new(&args[1]);
    match ledger.claim_id(&args[2]) {
        Ok(LedgerClaim::Claimed) => std::process::exit(0),
        Ok(_) => std::process::exit(1),
        Err(_) => std::process::exit(2),
    }
}
